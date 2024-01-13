use std::{
    convert::Infallible,
    ops::{FromResidual, Try},
    process::{ExitCode, Termination},
};

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;

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

#[derive(Clone, Debug)]
pub struct Result<T, E = Error>(pub std::result::Result<T, E>);

impl<T> Result<T> {
    pub fn into_inner(self) -> std::result::Result<T, Error> {
        self.0
    }
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

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl<T, E> FromResidual<std::result::Result<Infallible, E>> for Result<T>
where
    E: Into<Error>,
{
    fn from_residual(residual: std::result::Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(error) => Self(Err(error.into())),
        }
    }
}

impl<T> Try for Result<T> {
    type Output = T;
    type Residual = std::result::Result<Infallible, Error>;

    fn from_output(output: Self::Output) -> Self {
        Self(Ok(output))
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(output) => std::ops::ControlFlow::Continue(output),
            Err(error) => std::ops::ControlFlow::Break(Err(error)),
        }
    }
}

impl<T> Termination for Result<T> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(_) => ExitCode::SUCCESS,
            Err(error) => {
                tracing::error!("Program exit with error: {}", error);
                ExitCode::FAILURE
            }
        }
    }
}

impl<T> From<T> for Result<T> {
    fn from(t: T) -> Self {
        Self(Ok(t))
    }
}

impl<T, E: Into<Error>> From<std::result::Result<T, E>> for Result<T> {
    fn from(res: std::result::Result<T, E>) -> Self {
        match res {
            Ok(t) => Self(Ok(t)),
            Err(e) => Self(Err(e.into())),
        }
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
