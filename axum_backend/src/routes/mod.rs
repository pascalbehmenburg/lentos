mod user;
use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Extension};
use axum_session::{
    Key, SameSite, SessionConfig, SessionLayer, SessionMode, SessionPgPool, SessionStore,
};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use hyper::StatusCode;
use sqlx::{PgPool, Pool, Postgres};
use tower_http::compression::CompressionLayer;

use crate::{
    config::BackendConfig,
    response_error,
    routes::user::{PostgresUserRepository, User},
};
use crate::{error::Result, internal_error};

#[derive(Debug, Clone)]
pub struct Router {
    axum_router: axum::Router,
}

impl Router {
    pub async fn new(backend_config: Arc<BackendConfig>) -> Result<Self> {
        async fn handler_404() -> impl IntoResponse {
            response_error!(StatusCode::NOT_FOUND, "Error 404: Not found")
        }

        async fn handler_index() -> impl IntoResponse {
            "Hello World!"
        }

        let postgres_pool =
            Pool::<Postgres>::connect(&backend_config.database_url).await.map_err(|e| {
                internal_error!(
                    "Failed to connect to database at {}. Details: {}",
                    backend_config.database_url,
                    e
                )
            })?;

        let user_repo = Arc::new(PostgresUserRepository::new(postgres_pool.clone()));

        let axum_router = axum::Router::new()
            .layer(CompressionLayer::new().br(true).compress_when(|_, _, _: &_, _: &_| true))
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(Extension(user_repo.clone()))
            .layer(SessionLayer::new(
                SessionStore::<SessionPgPool>::new(
                    Some(postgres_pool.clone().into()),
                    SessionConfig::default()
                        .with_table_name("sessions")
                        .with_key(Key::from(backend_config.session_key.as_bytes()))
                        .with_database_key(Key::generate())
                        .with_http_only(true)
                        .with_secure(true)
                        .with_security_mode(axum_session::SecurityMode::PerSession)
                        .with_mode(SessionMode::Persistent)
                        .with_cookie_path("/")
                        .with_cookie_same_site(SameSite::Strict)
                        .with_bloom_filter(true),
                )
                .await
                .map_err(|e| internal_error!("Failed to create session store. Details: {}", e))?,
            ))
            .layer(
                AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(Some(
                    postgres_pool.clone(),
                ))
                .with_config(AuthConfig::<i64>::default().set_cache(true)),
            )
            .route("/", get(handler_index))
            .fallback(handler_404);
        Self { axum_router }.into()
    }

    pub fn into_inner(self) -> axum::Router {
        self.axum_router
    }
}
