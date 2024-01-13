use std::pin::Pin;

use actix_http::{HttpMessage, Payload, StatusCode};
use actix_identity::Identity;
use actix_web::{FromRequest, HttpRequest};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use futures_core::Future;
use rand::rngs::OsRng;
use shared::models::user::{SignInUser, User};

use super::api::user::UserError;
use crate::util::{error::Error, error_or::ErrorOr};
// TODO maybe create auth trait
impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let identity = Identity::from_request(req, &mut Payload::None);

        let future = async move {
            let identity = identity.await.map_err(|_| {
                Error::External(
                    StatusCode::UNAUTHORIZED,
                    "You do not seem to be logged in. Please log in first.".into(),
                )
            })?;
            let id = Self::parse_identity_id(identity).await?;
            Ok(Self { id })
        };

        Pin::from(Box::new(future))
    }
}
