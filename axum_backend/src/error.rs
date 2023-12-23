use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}, {1}")]
    ResponseError(StatusCode, String),

    #[error("{0}")]
    InternalError(String),

    // fallback error which is treated like an internal error
    #[error(transparent)]
    Other(#[from] color_eyre::eyre::Error),
}

#[macro_export]
macro_rules! internal_error {
    ($msg:literal $(,)?) => {
        $crate::Error::InternalError(format!($msg))
    };
    ($err:expr $(,)?) => {
        $crate::Error::from($err)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::InternalError(format!($fmt, $($arg)*))
    };
}

// Use this macro to return an explicit error for the user
#[macro_export]
macro_rules! response_error {
    ($status:expr, $msg:literal $(,)?) => {
        $crate::Error::ResponseError($status, format!($msg))
    };
    ($status:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::Error::ResponseError($status, format!($fmt, $($arg)*))
    };
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct ErrorMessage {
            error: String,
        }

        match self {
            Error::ResponseError(status_code, error_message) => {
                let json = serde_json::to_value(ErrorMessage { error: error_message }).unwrap();

                tracing::info!("Error: {}", json);

                (status_code, Json::<serde_json::Value>(json)).into_response()
            }
            _ => {
                let json = serde_json::to_value(ErrorMessage {
                    error: "Something went wrong! We're working on it.".into(),
                })
                .unwrap();

                tracing::error!("Error: {}", self);

                (StatusCode::INTERNAL_SERVER_ERROR, Json::<serde_json::Value>(json)).into_response()
            }
        }
    }
}
