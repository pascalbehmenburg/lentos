use std::ops::Deref;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
};
use hyper::StatusCode;
use tower_http::compression::CompressionLayer;

use crate::response_error;

#[derive(Debug, Clone)]
pub struct Router(axum::Router);

impl Router {
    #[must_use]
    pub fn new() -> Self {
        Self(
            axum::Router::new()
                .layer(CompressionLayer::new().br(true).compress_when(|_, _, _: &_, _: &_| true))
                .layer(tower_http::trace::TraceLayer::new_for_http()),
        )
    }

    pub fn with_routes(mut self) -> Self {
        async fn handler() -> Html<&'static str> {
            Html("<h1>Hello, World!</h1>")
        }

        async fn handler_404() -> impl IntoResponse {
            response_error!(StatusCode::NOT_FOUND, "Error 404: Not found")
        }

        self.0 = self.0.route("/", get(handler)).fallback(handler_404);
        self
    }

    pub fn into_inner(self) -> axum::Router {
        self.0
    }
}
