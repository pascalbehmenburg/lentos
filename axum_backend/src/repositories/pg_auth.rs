use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use hyper::StatusCode;
use shared::models::user::SignInUser;

use super::{pg_user_repo::PostgresUserRepository, UserRepository};
use crate::{internal_error, response_error, routes::User};

pub type AuthSession = axum_login::AuthSession<PostgresUserRepository>;

#[async_trait]
impl AuthnBackend for PostgresUserRepository {
    type User = User;
    type Credentials = SignInUser;
    type Error = crate::error::Error;

    async fn authenticate(
        &self,
        login_credentials: Self::Credentials,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        let user = self.get_by_email(&login_credentials.email).await?;

        // Check if provided credentials match the user with the provided email
        Argon2::default()
            .verify_password(
                login_credentials.password.as_bytes(),
                &PasswordHash::new(&user.password).map_err(|e| {
                    internal_error!("Failed to parse password hash from database. Details: {}", e)
                })?,
            )
            .map_err(|_| {
                response_error!(StatusCode::NOT_FOUND, "User with provided data does not exist.")
            })?;

        Ok(Some(user))
    }

    async fn get_user(
        &self,
        id: &UserId<Self>,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.get_by_id(id).await?))
    }
}
