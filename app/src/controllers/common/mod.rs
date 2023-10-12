use std::pin::Pin;

use actix_http::{HttpMessage, Payload, StatusCode};
use actix_identity::Identity;
use actix_web::{FromRequest, HttpRequest};
use argon2::{
    password_hash::SaltString, Argon2, PasswordHash, PasswordHasher,
    PasswordVerifier,
};

use futures_core::Future;
use rand::rngs::OsRng;
use shared::models::user::{LoginUser, User};

use crate::util::{error::Error, error_or::ErrorOr};

use super::api::user::UserError;
// TODO maybe create auth trait

/// Logs in a user by verifying their password and setting a session cookie.
///
/// # Arguments
///
/// * `request` - The HTTP request object.
/// * `db_user` - The user object from the database.
/// * `req_user` - The user object from the request.
///
/// # Errors
///
/// Returns an error if the password is invalid or if there is an internal
/// error.
pub async fn login(
    request: &HttpRequest,
    db_user: &User,
    req_user: &LoginUser,
) -> ErrorOr<()> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&db_user.password)?;

    argon2
        .verify_password(req_user.password.as_bytes(), &parsed_hash)
        .map_err(|_| UserError::InvalidEmailOrPassword)?;

    Identity::login(&request.extensions(), db_user.id.to_string())
        .map(|_| ())
        .map_err(Into::into)
        .map_err(Error::Internal)
        .into()
}

/// Hashes a password using the Argon2 password hashing algorithm.
///
/// # Arguments
///
/// * `password` - A string slice containing the password to hash.
///
/// # Returns
///
/// Returns a `Result` containing a `String` with the hashed password if
/// successful, or an `Error` if the password hashing failed.
pub async fn hash_password(password: &str) -> ErrorOr<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    password_hash.to_string().into()
}

pub struct AuthUser {
    pub id: i64,
}
impl AuthUser {
    async fn parse_identity_id(identity: Identity) -> ErrorOr<i64> {
        let identity_id =
            identity.id().map_err(|e| Error::Internal(e.into()))?;

        identity_id.parse::<i64>().map_err(|e| Error::Internal(e.into())).into()
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let identity = Identity::from_request(req, &mut Payload::None);

        let future = async move {
            let identity = identity.await.map_err(|_| {
                Error::External(
                    StatusCode::UNAUTHORIZED,
                    "You do not seem to be logged in. Please log in first."
                        .into(),
                )
            })?;
            let id = Self::parse_identity_id(identity).await?;
            Ok(Self { id })
        };

        Pin::from(Box::new(future))
    }
}
