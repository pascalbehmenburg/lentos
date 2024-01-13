use std::io::{BufReader, Error, ErrorKind};

use actix_identity::IdentityMiddleware;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    SessionMiddleware,
};
use actix_web::{
    cookie::{Key, SameSite},
    middleware::{self, Compat},
    App, HttpServer,
};
use app::{
    controllers::{self},
    repository::{
        session::PostgresSessionRepository,
        todo::{self},
        user,
    },
};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::pkcs8_private_keys;
use sqlx::{Pool, Postgres};
use tracing::subscriber::set_global_default;
use tracing_subscriber::Registry;

#[macro_use]
extern crate dotenv_codegen;

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_log::LogTracer;
    use tracing_subscriber::filter::*;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    LogTracer::init().expect("Failed to set logger");

    let fmt_layer = fmt::layer().with_target(true).pretty();

    let lib_filter_layer = Targets::new()
        .with_target("h2", LevelFilter::ERROR)
        .with_target("hyper", LevelFilter::ERROR)
        .with_default(LevelFilter::DEBUG);

    let subscriber =
        Registry::default().with(lib_filter_layer).with(fmt_layer).with(ErrorLayer::default());

    set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    install_tracing();
    color_eyre::install().unwrap();
    let pool = Pool::<Postgres>::connect(dotenv!("DATABASE_URL")).await.unwrap();

    HttpServer::new(move || {
        let todo_repository = todo::PostgresTodoRepository::new(pool.clone());
        let todo_repository = actix_web::web::Data::new(todo_repository);

        let user_repository = user::PostgresUserRepository::new(pool.clone());
        let user_repository = actix_web::web::Data::new(user_repository);

        let session_repository = PostgresSessionRepository::new(pool.clone());

        let cookie_priv_key = Key::from(dotenv!("SIGNING_KEY").as_bytes());

        App::new()
            .wrap(Compat::new(middleware::Logger::default()))
            .wrap(Compat::new(middleware::Compress::default()))
            .wrap(Compat::new(
                IdentityMiddleware::builder()
                    //.visit_deadline(Some(Duration::from_secs(config.cookie_timeout)))
                    .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
                    .build(),
            ))
            .wrap(Compat::new(
                SessionMiddleware::builder(session_repository, cookie_priv_key)
                    .session_lifecycle(PersistentSession::default())
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_same_site(SameSite::Strict)
                    .cookie_path("/".into())
                    .cookie_domain(None)
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .build(),
            ))
            .app_data(todo_repository)
            .app_data(user_repository)
            .configure(controllers::api::service)
    })
    .bind_rustls("127.0.0.1:8443", rustls_setup())
    .map_err(|e| Error::new(ErrorKind::Other, e))?
    .run()
    .await
}

fn rustls_setup() -> ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder().with_safe_defaults().with_no_client_auth();

    let cert_path = dotenv!("CERT_PATH");
    let key_path = dotenv!("KEY_PATH");

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(std::fs::File::open(cert_path).unwrap());
    let key_file = &mut BufReader::new(std::fs::File::open(key_path).unwrap());

    // convert files to key/cert objects
    let cert_chain =
        rustls_pemfile::certs(cert_file).unwrap().into_iter().map(Certificate).collect();
    let mut keys: Vec<PrivateKey> =
        pkcs8_private_keys(key_file).unwrap().into_iter().map(PrivateKey).collect();

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
