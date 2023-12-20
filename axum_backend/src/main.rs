use axum::{
    extract::{Request, Host},
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router, handler::HandlerWithoutStateExt,
};
use futures_util::pin_mut;
use hyper::{body::Incoming, StatusCode, Uri};
use hyper_util::rt::{TokioExecutor, TokioIo};
use listenfd::ListenFd;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc, panic::panic_any, net::SocketAddr};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower_service::Service;

use color_eyre::eyre::{WrapErr, Result};

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    const IP_ADDRESS: &str = "127.0.0.1";

    let ports = Ports {
        http: 8080,
        https: 8443,
    };

    // init tracing (logs) and error handling (color_eyre)
    {
        use tracing::subscriber::set_global_default;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::filter::*;
        use tracing_subscriber::fmt;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::Registry;

        let fmt_layer = fmt::layer().with_target(true).pretty();

        let lib_filter_layer = Targets::new()
            .with_target("h2", LevelFilter::ERROR)
            .with_target("hyper", LevelFilter::ERROR)
            .with_target("axum::rejection", LevelFilter::TRACE)
            .with_target("tower_http", LevelFilter::DEBUG)
            .with_default(LevelFilter::DEBUG);

        let subscriber = Registry::default()
            .with(lib_filter_layer)
            .with(fmt_layer)
            .with(ErrorLayer::default());

        set_global_default(subscriber)
            .wrap_err("Failed to set tracing subscriber.")?;
        color_eyre::install()
            .wrap_err("Failed to install color_eyre error handler.")?;
        tracing::info!("Tracing setup complete.");
    }

    // this sets up the tls config
    let rustls_config = {
        let cert_file_path = PathBuf::from("127.0.0.1+1.pem");
        let mut cert_reader = BufReader::new(
            File::open(cert_file_path).wrap_err("Certificate file not found.")?,
        );

        let key_file_path = PathBuf::from("127.0.0.1+1-key.pem");
        let mut key_reader = BufReader::new(
            File::open(key_file_path).wrap_err("Key file not found.")?,
        );

        let key = PrivateKey(
            pkcs8_private_keys(&mut key_reader)
                .wrap_err("Failed to construct pkcs8 private key. Check the key file.")?
                .remove(0),
        );

        let certs = certs(&mut cert_reader)
            .wrap_err("Failed to construct certs. Check the certificates.")?
            .into_iter()
            .map(Certificate)
            .collect();

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .wrap_err("Failed to construct tls server config. Check the cert / key files.")?;

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        tracing::info!("TLS setup completed with config: {:?}", config);

        Arc::new(config)
    };

    // http to https redirect service
    tokio::spawn(async move {
        fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri> {
            let mut parts = uri.into_parts();

            parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

            if parts.path_and_query.is_none() {
                parts.path_and_query = Some("/".parse().unwrap());
            }

            let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
            parts.authority = Some(https_host.parse()?);

            Ok(Uri::from_parts(parts)?)
        }

        let redirect = move |Host(host): Host, uri: Uri| async move {
            match make_https(host, uri, ports) {
                Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
                Err(error) => {
                    tracing::warn!(%error, "failed to convert URI to HTTPS");
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        };

        let mut listenfd = ListenFd::from_env();
        let tcp_listener = match listenfd
            .take_tcp_listener(0)
            .expect("fd at position 0 was not an tcp listener")
            {
                Some(tcp_listener) => TcpListener::from_std(tcp_listener).expect(""),
                None => TcpListener::bind(format!("{}:{}", IP_ADDRESS, ports.http)).await.unwrap_or_else(|_| {
                    panic_any(color_eyre::eyre::eyre!("failed to bind systemfd tcp listener to adress using fallback: {:?}:{:?}", IP_ADDRESS, ports.http))
                }),
            };
        tracing::info!("Listening to http requests from {}", tcp_listener.local_addr().expect("Failed to get address of tcp listener"));
        axum::serve(tcp_listener, redirect.into_make_service())
            .await
            .expect("Failed to serve http to https redirect service.");
    });

    let mut listenfd = ListenFd::from_env();
    let tcp_listener = match listenfd
        .take_tcp_listener(1)
        .wrap_err("fd at position 1 was not an tcp listener")?
    {
        Some(tcp_listener) => TcpListener::from_std(tcp_listener).expect(""),
        None => TcpListener::bind(format!("{}:{}", IP_ADDRESS, ports.https)).await.unwrap_or_else(|_| {
            panic_any(color_eyre::eyre::eyre!("failed to bind systemfd tcp listener to adress using fallback: {:?}", IP_ADDRESS))
        }),
    };

    let app_router = Router::new().route("/", get(handler));
    let app_router = app_router.fallback(handler_404);
    let app_router = app_router.layer(tower_http::trace::TraceLayer::new_for_http());

    // we bind this here since rustls_config doesnt implement clone and we don't want to rebuild it
    let tls_acceptor = TlsAcceptor::from(rustls_config);
    pin_mut!(tcp_listener);
    loop {
        let app_service = app_router.clone();
        let tls_acceptor = tls_acceptor.clone();

        let (conn, addr) = tcp_listener.accept().await.wrap_err("Failed to accept tcp connection.")?;

        tokio::spawn(async move {
            // wait for tls handshake to happen
            let Ok(stream) = tls_acceptor.accept(conn).await else {
                tracing::info!("Failed tls handshake with: {:?}", addr);
                return;
            };

            // convert the tokio stream to a hyper stream and forward the incoming requests to our axum app
            let stream = TokioIo::new(stream);
            let hyper_service = hyper::service::service_fn(
                move |request: Request<Incoming>| {
                    app_service.clone().call(request)
                },
            );

            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await
                .expect("Failed to serve connection.");
        });
    }
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Error 404: Not found")
}