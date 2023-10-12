use actix_http::StatusCode;
use color_eyre::eyre;

use crate::util::error::Error;

#[derive(Debug, derive_more::Display)]
/// Enum to convert HTTP methods to more user-friendly CRUD op's.
pub enum Operation {
    #[display(fmt = "create")]
    Post,

    #[display(fmt = "receive")]
    Receive,

    #[display(fmt = "update")]
    Update,

    #[display(fmt = "delete")]
    Delete,
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum RepositoryError {
    #[display(fmt = "{} was not found", relation_name)]
    NotFound {
        relation_name: String,
    },

    #[display(
        fmt = "You have no permission to {} this {}",
        operation,
        relation_name
    )]
    Forbidden {
        operation: Operation,
        relation_name: String,
    },

    Internal(#[error(not(source))] eyre::Error),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound { .. } => {
                Error::External(StatusCode::NOT_FOUND, error.to_string().into())
            }
            RepositoryError::Forbidden { .. } => {
                Error::External(StatusCode::FORBIDDEN, error.to_string().into())
            }
            RepositoryError::Internal(error) => Error::Internal(error),
        }
    }
}
