use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // used for errors which should be displayed in a response
    #[error("{0}, {1}")]
    ResponseError(StatusCode, String),

    // used for errors which should not be displayed in a response
    #[error("{0}")]
    InternalError(String),

    // fallback error
    #[error(transparent)]
    Other(#[from] color_eyre::eyre::Error),
}

// We use a JSON response so that the frontend may re-use ResponseError
// messages. Otherwise we just return a generic error message.
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

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::InternalError(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::InternalError(err.to_string())
    }
}

// Use this macro to return an any error which should not be displayed in a
// response
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
