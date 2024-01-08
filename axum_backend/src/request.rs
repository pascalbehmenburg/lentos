use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    Form, Json, RequestExt,
};
use hyper::{header::CONTENT_TYPE, StatusCode};

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

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
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
    use std::sync::Arc;

    use axum::{body::Body, http, routing::post};
    use tower::ServiceExt;

    use super::*;
    use crate::{config::BackendConfig, routes::Router};

    #[tokio::test]
    async fn test_json_or_form_extractor() {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Payload {
            name: String,
        }

        let config = Arc::new(BackendConfig::load().await.into_inner().unwrap());
        let router = Router::new(config).await.into_inner().unwrap();
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
