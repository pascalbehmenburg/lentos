use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Extension, Json};
use hyper::StatusCode;
use shared::models::user::User;
use sqlx::{Pool, Postgres};
use tower_http::compression::CompressionLayer;

use crate::{
    config::BackendConfig,
    repositories::{PostgresUserRepository, UserRepository},
    response_error,
};
use crate::{error::Result, internal_error};

#[derive(Debug, Clone)]
pub struct Router {
    router: axum::Router,
    backend_config: Arc<BackendConfig>,
}

impl Router {
    pub fn new(backend_config: Arc<BackendConfig>) -> Self {
        Self {
            router: axum::Router::new()
                .layer(CompressionLayer::new().br(true).compress_when(|_, _, _: &_, _: &_| true))
                .layer(tower_http::trace::TraceLayer::new_for_http()),
            backend_config,
        }
    }

    pub async fn with_routes(mut self) -> Result<Self> {
        async fn handler(db: Extension<Arc<PostgresUserRepository>>) -> Result<Json<User>> {
            let user = db.get_session_user(&3).await?;

            Ok(Json(user))
        }

        async fn handler_404() -> impl IntoResponse {
            response_error!(StatusCode::NOT_FOUND, "Error 404: Not found")
        }
        let postgres_pool =
            Pool::<Postgres>::connect(&self.backend_config.database_url).await.map_err(|e| {
                internal_error!(
                    "Failed to connect to database at {}. Details: {}",
                    self.backend_config.database_url,
                    e
                )
            })?;

        let user_db = Arc::new(PostgresUserRepository::new(postgres_pool.clone()));

        self.router =
            self.router.route("/", get(handler)).layer(Extension(user_db)).fallback(handler_404);
        Ok(self)
    }

    pub fn into_inner(self) -> axum::Router {
        self.router
    }
}
