use std::panic::panic_any;
use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use axum::{response::IntoResponse, Extension, Json};
use axum_session::SessionPgPool;
use axum_session_auth::{AuthSession, Authentication};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use shared::models::user::SignInUser;
use shared::models::user::{CreateUser, UpdateUser};
use sqlx::{types::chrono, PgPool};

use crate::response_error;
use crate::{error::Result, internal_error};

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

#[async_trait]
impl Authentication<User, i64, PgPool> for User {
    async fn load_user(
        userid: i64,
        pool: Option<&PgPool>,
    ) -> std::result::Result<User, anyhow::Error> {
        let user_repo = PostgresUserRepository {
            pool: pool.ok_or_else(|| anyhow::anyhow!("Failed to get database pool."))?.clone(),
        };
        user_repo.get_by_id(&userid).await.into()
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

pub trait UserRepository: Send + Sync + 'static {
    async fn get_by_id(&self, session_user_id: &i64) -> Result<User>;

    async fn get_by_email(&self, email: &str) -> Result<User>;

    async fn create(&self, create_user: &CreateUser) -> Result<()>;

    async fn update(&self, update_user: &UpdateUser, session_user_id: &i64) -> Result<()>;

    async fn delete(&self, session_user_id: &i64) -> Result<()>;

    async fn create_table(&self) -> Result<()>;
}

pub struct PostgresUserRepository {
    pub pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let user_repo = Self { pool: pool.clone() };
        tokio::spawn(async move {
            Self { pool }.create_table().await.into_inner().unwrap_or_else(|e| {
                panic_any(internal_error!("Failed to create user tables. Details: {}", e))
            });
        });
        user_repo
    }
}

impl UserRepository for PostgresUserRepository {
    async fn create_table(&self) -> Result<()> {
        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    anonymous BOOLEAN NOT NULL DEFAULT true,
                    name VARCHAR(256) NOT NULL,
                    email VARCHAR(256) NOT NULL UNIQUE,
                    password VARCHAR(256) NOT NULL,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| internal_error!("Failed to create user table. Details: {}", e))?;

        // inserts a guest user to id 1 and ensures its name and anonymity are set
        // correctly. It is used for the auth middleware to check if a user is
        // authenticated or not.
        sqlx::query(
            r#"
                INSERT INTO users
                    (id, anonymous, name)
                SELECT 1, true, 'Guest'
                ON CONFLICT(id) DO UPDATE SET
                    anonymous = EXCLUDED.anonymous,
                    username = EXCLUDED.username
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            internal_error!("Failed to insert guest user to id 1 in users table. Details: {}", e)
        })?;

        Result(Ok(()))
    }

    async fn get_by_email(&self, email: &str) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::NOT_FOUND, "User with provided data does not exist.")
            }
            _ => internal_error!("Failed to get user by email. Details: {}", e),
        })
        .into()
    }

    async fn get_by_id(&self, user_id: &i64) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| internal_error!("Failed to get user by id. Details: {}", e))
        .into()
    }

    async fn create(&self, create_user: &CreateUser) -> Result<()> {
        sqlx::query(
            r#"
            INSERT
            INTO users (name, email, password)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&create_user.name)
        .bind(&create_user.email)
        .bind(&create_user.password)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::Database(e) => match e.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    response_error!(
                        StatusCode::CONFLICT,
                        "A user with the provided email address already exists. Please choose \
                         another one."
                    )
                }
                sqlx::error::ErrorKind::NotNullViolation => {
                    response_error!(StatusCode::BAD_REQUEST, "Please provide all required fields.")
                }
                sqlx::error::ErrorKind::CheckViolation => {
                    response_error!(
                        StatusCode::BAD_REQUEST,
                        "It seemed like the data was in a wrong format. Please check if you \
                         provided the correct data."
                    )
                }
                _ => internal_error!("Failed to create user. Details: {}", e),
            },
            _ => internal_error!("Failed to create user. Details: {}", e),
        })
        .into()
    }

    async fn update(&self, update_user: &UpdateUser, user_id: &i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET
                name = COALESCE($1, name),
                email = COALESCE($2, email),
                password = COALESCE($3, password),
                updated_at = now()
            WHERE id = $4
            "#,
        )
        .bind::<&Option<String>>(&update_user.name)
        .bind::<&Option<String>>(&update_user.email)
        .bind::<&Option<String>>(&update_user.password)
        .bind::<&i64>(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) => match db_err.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    response_error!(
                        StatusCode::CONFLICT,
                        "A user with the provided email address already exists. Please choose \
                         another one."
                    )
                }
                sqlx::error::ErrorKind::NotNullViolation => {
                    response_error!(StatusCode::BAD_REQUEST, "Please provide all required fields.")
                }
                sqlx::error::ErrorKind::CheckViolation => {
                    response_error!(
                        StatusCode::BAD_REQUEST,
                        "It seemed like the data was in a wrong format. Please check if you \
                         provided the correct data."
                    )
                }
                _ => internal_error!("Failed to update user. Details: {}", db_err),
            },
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::FORBIDDEN, "You are not permitted to modify this user.")
            }
            _ => internal_error!("Failed to update user. Details: {}", err),
        })
        .into()
    }

    async fn delete(&self, user_id: &i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::FORBIDDEN, "You are not permitted to delete this user.")
            }
            _ => internal_error!(e),
        })
        .into()
    }
}

async fn login<R: UserRepository>(
    Json(req_user): Json<SignInUser>,
    auth: AuthSession<User, i64, SessionPgPool, PgPool>,
    Extension(repo): Extension<Arc<PostgresUserRepository>>,
) -> Result<impl IntoResponse> {
    let auth_user = auth.current_user.clone().unwrap_or_default();

    if !auth_user.is_anonymous() {
        return Result(Err(response_error!(
            StatusCode::BAD_REQUEST,
            "You seem to be logged in already. Try signing out first."
        )));
    };

    let db_user = repo.get_by_email(&req_user.email).await?;

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&db_user.password).map_err(|e| {
        internal_error!("Failed to parse password hash from database. Details: {}", e)
    })?;

    argon2.verify_password(req_user.password.as_bytes(), &parsed_hash).map_err(|_| {
        response_error!(StatusCode::NOT_FOUND, "User with provided data does not exist.")
    })?;

    auth.login_user(db_user.id);
    auth.remember_user(true);

    StatusCode::OK.into()
}
// async fn login<R: UserRepository>(
//     request: HttpRequest,
//     login_user: web::Json<SignInUser>,
//     repo: web::Data<R>,
// ) -> ErrorOr<HttpResponse> {
//     let user = repo
//         .get_user_by_email(&login_user.email)
//         .await
//         .0
//         .map_err(|_| UserError::InvalidEmailOrPassword)?;

//     common::login(&request, &user, &login_user).await?;

//     HttpResponse::Ok().finish().into()
// }

// async fn register<R: UserRepository>(
//     mut create_user: web::Json<CreateUser>,
//     repo: web::Data<R>,
// ) -> ErrorOr<HttpResponse> {
//     create_user.password =
// common::hash_password(&create_user.password).await?;

//     repo.create_user(&create_user).await?;

//     HttpResponse::Ok().finish().into()
// }

// async fn get<R: UserRepository>(
//     repo: web::Data<R>,
//     user: AuthUser,
// ) -> ErrorOr<Json<User>> {
//     repo.get_session_user(&user.id).await.0.map(Json).into()
// }

// async fn put<R: UserRepository>(
//     update_user: web::Json<UpdateUser>,
//     repo: web::Data<R>,
//     user: AuthUser,
// ) -> ErrorOr<HttpResponse> {
//     repo.update_user(&update_user, &user.id).await?;

//     HttpResponse::Ok().finish().into()
// }

// async fn delete<R: UserRepository>(
//     repo: web::Data<R>,
//     user: AuthUser,
// ) -> ErrorOr<HttpResponse> {
//     repo.delete_user(&user.id).await?;

//     HttpResponse::Ok().finish().into()
// }
