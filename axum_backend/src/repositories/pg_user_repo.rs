use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use hyper::StatusCode;
use shared::models::user::{CreateUser, SignInUser, UpdateUser};

use super::UserRepository;
use crate::{error::Result, internal_error, response_error, routes::User};

#[derive(Debug, Clone)]
pub(crate) struct PostgresUserRepository {
    pub pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let user_repo = Self { pool: pool.clone() };
        tokio::spawn(async move {
            Self { pool }
                .create_table()
                .await
                .into_inner()
                .map_err(|e| internal_error!("Failed to create user tables. Details: {}", e))
                .unwrap_or(());
        });
        user_repo
    }

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

        if !Self::get_by_id(&self, &1).await.0.is_ok() {
            crate::routes::user::register(
                CreateUser::new("Guest", "guest@guest.com", "Guest"),
                self.clone(),
            );
        } else {
        }

        // inserts a guest user to id 1 and ensures its name and anonymity are set
        // correctly. It is used for the auth middleware to check if a user is
        // authenticated or not.

        Result(Ok(()))
    }
}

impl UserRepository for PostgresUserRepository {
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

    async fn get_by_id(&self, id: &i64) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| internal_error!("Failed to get user by id. Details: {}", e))
        .into()
    }

    async fn create(&self, user: &CreateUser) -> Result<()> {
        sqlx::query(
            r#"
            INSERT
            INTO users (name, email, password)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::Database(e) => match e.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    response_error!(
                        StatusCode::CONFLICT,
                        "A user with the provided email address already exists."
                    )
                }
                sqlx::error::ErrorKind::NotNullViolation => {
                    response_error!(StatusCode::BAD_REQUEST, "Not all required fields provided.")
                }
                sqlx::error::ErrorKind::CheckViolation => {
                    response_error!(
                        StatusCode::BAD_REQUEST,
                        "The provided data was in a format the server could not process."
                    )
                }
                _ => internal_error!("Failed to create user. Details: {}", e),
            },
            _ => internal_error!("Failed to create user. Details: {}", e),
        })
        .into()
    }

    async fn update(&self, user: &UpdateUser, id: &i64) -> Result<()> {
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
        .bind::<&Option<String>>(&user.name)
        .bind::<&Option<String>>(&user.email)
        .bind::<&Option<String>>(&user.password)
        .bind::<&i64>(id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|err| match err {
            sqlx::Error::Database(db_err) => match db_err.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    response_error!(
                        StatusCode::CONFLICT,
                        "A user with the provided email address already exists."
                    )
                }
                sqlx::error::ErrorKind::NotNullViolation => {
                    response_error!(StatusCode::BAD_REQUEST, "Please provide all required fields.")
                }
                sqlx::error::ErrorKind::CheckViolation => {
                    response_error!(
                        StatusCode::BAD_REQUEST,
                        "The provided data was in a format the server could not process."
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

    async fn delete(&self, id: &i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
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
