use std::{fs::File, io::BufReader, panic::panic_any, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::{FromRequest, Host, Request},
    handler::HandlerWithoutStateExt,
    response::{IntoResponse, Redirect, Response},
    Form, Json, RequestExt,
};
use futures_util::pin_mut;
use hyper::{body::Incoming, header::CONTENT_TYPE, StatusCode, Uri};
use hyper_util::rt::{TokioExecutor, TokioIo};
use listenfd::ListenFd;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};

mod config;
mod error;
mod routes;
use config::BackendConfig;
pub use error::Error;
use error::Result;
use routes::Router;
use tower_service::Service;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::*;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<()> {
    // init tracing (logs) and error handling (color_eyre)
    {
        let fmt_layer = fmt::layer().with_target(true).pretty();

        let lib_filter_layer = Targets::new()
            .with_target("h2", LevelFilter::ERROR)
            .with_target("hyper", LevelFilter::ERROR)
            .with_target("axum::rejection", LevelFilter::TRACE)
            .with_target("tower_http", LevelFilter::DEBUG)
            .with_default(LevelFilter::DEBUG);

        let subscriber =
            Registry::default().with(lib_filter_layer).with(fmt_layer).with(ErrorLayer::default());

        set_global_default(subscriber)
            .map_err(|_| internal_error!("Failed to set global tracing subscriber"))?;
        color_eyre::install()
            .map_err(|_| internal_error!("Failed to install color_eyre error handler."))?;
        tracing::info!("Tracing setup complete.");
    }

    let config = BackendConfig::load().await?;

    // this sets up the tls config
    let rustls_config = {
        let mut cert_reader = BufReader::new(
            File::open(config.cert_file_path)
                .map_err(|_| internal_error!("Certificate file not found."))?,
        );

        let mut key_reader = BufReader::new(
            File::open(config.key_file_path).map_err(|_| internal_error!("Key file not found."))?,
        );

        let key = PrivateKey(
            pkcs8_private_keys(&mut key_reader)
                .map_err(|e| {
                    internal_error!("Failed to construct pkcs8 private key. Details: {}", e)
                })?
                .remove(0),
        );

        let cert = certs(&mut cert_reader)
            .map_err(|e| internal_error!("Failed to construct certs. {}", e))?
            .into_iter()
            .map(Certificate)
            .collect();

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .map_err(|e| {
                internal_error!(
                    "Invalid private key provided when building rustls config. Details: {}",
                    e
                )
            })?;

        config.alpn_protocols = vec![b"h2".to_vec()];

        tracing::info!("TLS setup completed with config: {:?}", config);

        Arc::new(config)
    };

    // used to create tcp listeners for http and https connections
    async fn create_tcp_listener<'a>(
        ip_address: &'a str,
        port: &'a usize,
        listenfd_idx: usize,
    ) -> Result<TcpListener> {
        let mut listenfd = ListenFd::from_env();
        let tcp_listener = match listenfd.take_tcp_listener(listenfd_idx).unwrap_or_else(|_| {
            panic_any(internal_error!("fd at position 0 was not an tcp listener"))
        }) {
            Some(tcp_listener) => TcpListener::from_std(tcp_listener).expect(""),
            None => TcpListener::bind(format!("{}:{}", ip_address, port)).await.map_err(|_| {
                internal_error!(
                    "Failed to bind systemfd tcp listener to address using fallback: {:?}:{:?}",
                    ip_address,
                    port
                )
            })?,
        };

        tracing::info!(
            "Listening to tcp connections on port {}",
            tcp_listener.local_addr().expect("Failed to get address of tcp listener")
        );

        Ok(tcp_listener)
    }

    // axum http to https redirect service
    let ip_address = config.ip_address.to_string();
    tokio::spawn(async move {
        let redirect = {
            let http_port = config.http_port.to_string();
            let https_port = config.https_port.to_string();

            move |Host(host): Host, uri: Uri| async move {
                // this closure ensures that http is upgraded to https
                let make_https = |host: String, uri: Uri| -> Result<Uri> {
                    let mut parts = uri.into_parts();

                    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

                    if parts.path_and_query.is_none() {
                        parts.path_and_query = Some("/".parse().unwrap());
                    }

                    let https_host = host.replace(&http_port, &https_port);

                    parts.authority = Some(https_host.parse().map_err(|e| {
                        internal_error!(
                            "Failed replacing the http URI host with https. Details: {}",
                            e
                        )
                    })?);

                    Uri::from_parts(parts).map_err(|e| {
                        internal_error!("Failed to construct URI from parts. Details: {}", e)
                    })
                };

                match make_https(host, uri) {
                    Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
                    Err(error) => Err(internal_error!(
                        "Failed to redirect from http to https. Details: {}",
                        error
                    )),
                }
            }
        };

        let tcp_listener =
            create_tcp_listener(&ip_address, &config.http_port, 0).await.unwrap_or_else(|_| {
                panic_any(internal_error!(
                    "Failed to get tcp listener for http to https redirect service."
                ))
            });

        axum::serve(tcp_listener, redirect.into_make_service())
            .await
            .expect("Failed to serve http to https redirect service.");
    });

    // main axum app
    let tcp_listener =
        create_tcp_listener(&config.ip_address, &config.https_port, 1).await.unwrap_or_else(|_| {
            panic_any(internal_error!(
                "Failed to get tcp listener for http to https redirect service."
            ))
        });

    let app_router = Router::new().with_routes();

    let tls_acceptor = TlsAcceptor::from(rustls_config);
    pin_mut!(tcp_listener);
    loop {
        let app_service = app_router.clone().into_inner();
        let tls_acceptor = tls_acceptor.clone();

        // accept tls conns
        let (conn, addr) = tcp_listener.accept().await.map_err(|e| {
            internal_error!("Failed to accept incoming tcp connection. Details: {}", e)
        })?;

        tokio::spawn(async move {
            // wait for tls handshake to happen
            let Ok(stream) = tls_acceptor.accept(conn).await else {
                tracing::info!("Failed tls handshake with: {:?}", addr);
                return;
            };

            // convert the tokio stream to a hyper stream and forward the
            // incoming requests to our axum app
            let stream = TokioIo::new(stream);
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                app_service.clone().call(request)
            });

            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await
                .expect("Failed to serve connection.");
        });
    }
}

struct JsonOrForm<T>(T);

#[async_trait]
impl<S, T> FromRequest<S> for JsonOrForm<T>
where
    S: Send + Sync,
    Json<T>: FromRequest<()>,
    Form<T>: FromRequest<()>,
    T: 'static,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = content_type_header.and_then(|value| value.to_str().ok());

        if let Some(content_type) = content_type {
            if content_type.starts_with(mime::APPLICATION_JSON.as_ref()) {
                let Json(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }

            if content_type.starts_with(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()) {
                let Form(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }
        }

        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response())
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http, routing::post};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_json_or_form_extractor() {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Payload {
            name: String,
        }

        let router = crate::routes::Router::new();
        let router = router.into_inner().route(
            "/test",
            post(|JsonOrForm(payload): JsonOrForm<Payload>| async move {
                (StatusCode::OK, format!("We got data: {payload:?}"));
            }),
        );

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/test")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(
                serde_json::to_string(&Payload { name: "MemeJson".to_string() }).unwrap(),
            ))
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();

        println!("{:?}", response);
        assert_eq!(response.status(), StatusCode::OK);

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/test")
            .header(http::header::CONTENT_TYPE, mime::APPLICATION_WWW_FORM_URLENCODED.as_ref())
            .body(Body::from(
                serde_urlencoded::to_string(Payload { name: "MemeForm".to_string() }).unwrap(),
            ))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        println!("{:?}", response);
        assert_eq!(response.status(), StatusCode::OK);
    }
}
