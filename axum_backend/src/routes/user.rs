use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{response::IntoResponse, Extension, Json};
use axum_login::AuthUser;
use hyper::StatusCode;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use shared::models::user::SignInUser;
use shared::models::user::{CreateUser, UpdateUser};
use sqlx::types::chrono;

use crate::{
    error::Result,
    internal_error,
    repositories::{pg_auth::AuthSession, UserRepository},
    response_error,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub anonymous: bool,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 1,
            anonymous: true,
            name: "Guest".to_string(),
            email: "".to_string(),
            password: "".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.password.as_bytes()
    }
}

/// Creates a user session when provided with valid login credentials and if
/// there is no existing one.
pub async fn login<UserRepo: UserRepository>(
    mut auth: AuthSession,
    Json(credentials): Json<SignInUser>,
) -> Result<impl IntoResponse> {
    let user = auth.authenticate(credentials).await.map_err(|e| internal_error!("{}", e))?;
    let user = user.ok_or_else(|| {
        response_error!(StatusCode::NOT_FOUND, "No match for provided credentials.")
    })?;

    auth.login(&user).await.map_err(|e| internal_error!("{}", e))?;

    StatusCode::OK.into()
}

pub async fn create<UserRepo: UserRepository>(
    Json(user): Json<CreateUser>,
    Extension(user_repo): Extension<Arc<UserRepo>>,
) -> Result<impl IntoResponse> {
    register(user, &*user_repo.clone()).await?;
    StatusCode::OK.into()
}

pub async fn register<UserRepo: UserRepository>(
    mut user: CreateUser,
    user_repo: &UserRepo,
) -> Result<()> {
    let salt = &SaltString::generate(&mut OsRng);
    let password_hash =
        Argon2::default().hash_password(user.password.as_bytes(), salt).map_err(|_| {
            response_error!(
                StatusCode::BAD_REQUEST,
                "Please try another password or reloading the application."
            )
        })?;

    // TODO perform sanitization

    // persist user data
    user.password = password_hash.to_string();
    user_repo.create(&user).await?;
    ().into()
}

pub async fn get<UserRepo: UserRepository>(
    auth: AuthSession,
    Extension(user_repo): Extension<Arc<UserRepo>>,
) -> Result<impl IntoResponse> {
    user_repo.get_by_id(&auth.user.unwrap().id).await.0.map(Json)?.into()
}

pub async fn put<UserRepo: UserRepository>(
    auth: AuthSession,
    Json(update_user): Json<UpdateUser>,
    Extension(user_repo): Extension<Arc<UserRepo>>,
) -> Result<impl IntoResponse> {
    user_repo.update(&update_user, &auth.user.unwrap().id).await?;
    StatusCode::OK.into()
}

async fn delete<UserRepo: UserRepository>(
    auth: AuthSession,
    Extension(user_repo): Extension<Arc<UserRepo>>,
) -> Result<impl IntoResponse> {
    user_repo.delete(&auth.user.unwrap().id).await?;
    StatusCode::OK.into()
}

#[cfg(test)]
mod tests {

    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::{config::BackendConfig, routes::Router};

    #[tokio::test]
    async fn login() {
        let router = Router::new(Arc::new(BackendConfig::load().await.0.unwrap())).await.0.unwrap();
        let login_credentials = SignInUser { email: "".into(), password: "".into() };

        let response = router
            .into_inner()
            .oneshot(
                Request::builder()
                    .uri("/login")
                    .method("POST")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&login_credentials).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:?}", response);
        println!("{:?}", response.body());
        assert_eq!(response.status(), StatusCode::OK);
    }
}
