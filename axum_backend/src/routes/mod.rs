pub(crate) mod user;
use std::sync::Arc;

use axum::{
    extract::Request,
    response::IntoResponse,
    routing::{get, post},
    Extension,
};
use axum_login::login_required;
use hyper::StatusCode;
use sqlx::{PgPool, Pool, Postgres};
use tower_http::compression::CompressionLayer;
use tower_sessions::{PostgresStore, SessionManagerLayer};

use crate::{
    config::BackendConfig, repositories::pg_user_repo::PostgresUserRepository, response_error,
};
use crate::{error::Result, internal_error};

#[derive(Debug, Clone)]
pub struct Router {
    axum_router: axum::Router,
}

impl Router {
    pub async fn new(backend_config: Arc<BackendConfig>) -> Result<Self> {
        // log the request method and path
        async fn handler_404(request: Request) -> impl IntoResponse {
            tracing::debug!("{request:?}");
            StatusCode::NOT_FOUND
        }

        async fn handler_index() -> impl IntoResponse {
            "Hello World!"
        }

        // setup database
        let postgres_pool =
            Pool::<Postgres>::connect(&backend_config.database_url).await.map_err(|e| {
                internal_error!(
                    "Failed to connect to database at {}. Details: {}",
                    backend_config.database_url,
                    e
                )
            })?;

        let user_repo = Arc::new(PostgresUserRepository::new(postgres_pool.clone()));

        // setup sessions
        let session_store = PostgresStore::new(postgres_pool.clone());

        session_store
            .migrate()
            .await
            .map_err(|e| internal_error!("Failed to perform session migrations. Details: {}", e))?;

        // let deletion_task = tokio::task::spawn(loop {
        //     session_store.clone().delete_expired().await.unwrap_or_else(|e| {
        //         tracing::error!("Failed to delete expired sessions. Details: {}", e);
        //     });
        //     tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
        // });

        // deletion_task.await?;

        // compose router
        let axum_router = axum::Router::new()
            .layer(CompressionLayer::new().br(true).compress_when(|_, _, _: &_, _: &_| true))
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(Extension(user_repo.clone()))
            .layer(SessionManagerLayer::new(session_store))
            .route("/", get(handler_index))
            .route("/users", get(user::get::<PostgresUserRepository>))
            .route_layer(login_required!(PostgresUserRepository, login_url = "/login"))
            .route("/login", post(user::login::<PostgresUserRepository>))
            .fallback(handler_404);
        Self { axum_router }.into()
    }

    pub fn into_inner(self) -> axum::Router {
        self.axum_router
    }
}
