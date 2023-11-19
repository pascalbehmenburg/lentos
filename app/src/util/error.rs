use std::borrow::Cow;

use actix_http::StatusCode;
use actix_web::{http::header::ContentType, HttpResponse, ResponseError};
use color_eyre::eyre;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum Error {
    #[display(fmt = "Error {}: {}", _0, _1)]
    External(StatusCode, Cow<'static, str>),

    #[display(fmt = "{}", _0)]
    Internal(#[error(not(source))] eyre::Error),
}

// TODO register custom errorhandler middleware to always respond with
// Error::Internal when there is a Error 500
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::External(status_code, _) => *status_code,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::External(status_code, error_message) => {
                HttpResponse::build(*status_code)
                    // we want an own content type so that we can differentiate
                    // between external errors that are
                    // produced by this very Error type and
                    // internal errors that are produced by actix that we need
                    // to map to an Error::Internal later
                    .content_type("ExternalError")
                    .body::<String>(error_message.to_string())
            }
            Error::Internal(error) => {
                tracing::error!("{:?}", error);
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type(ContentType::plaintext())
                    .body(format!(
                        r#"Status {}: We did a little wunky funky
                        and I cannot tell you what it is."#,
                        StatusCode::INTERNAL_SERVER_ERROR
                    ))
            }
        }
    }
}

impl From<eyre::Error> for Error {
    fn from(e: eyre::Error) -> Self {
        Error::Internal(e)
    }
}

impl From<argon2::Error> for Error {
    fn from(e: argon2::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(e: argon2::password_hash::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Error::External(
                StatusCode::NOT_FOUND,
                "The requested resource was not found".into(),
            ),
            _ => Error::Internal(e.into()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Internal(e.into())
    }
}
