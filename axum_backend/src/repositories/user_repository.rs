use async_trait::async_trait;
use hyper::StatusCode;
use shared::models::user::{CreateUser, UpdateUser, User};

use crate::response_error;
use crate::{error::Result, internal_error};

/// SAFETY: never expose get_user_by_email to an endpoint directly
/// also never state that an email exists or does not exist.
/// One may state that both the email and password combination are invalid
/// without leaking additional information.
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn get_session_user(&self, session_user_id: &i64) -> Result<User>;

    async fn get_user_by_email(&self, email: &str) -> Result<User>;

    async fn create_user(&self, create_user: &CreateUser) -> Result<()>;

    async fn update_user(&self, update_user: &UpdateUser, session_user_id: &i64) -> Result<()>;

    async fn delete_user(&self, session_user_id: &i64) -> Result<()>;
}

pub struct PostgresUserRepository {
    pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::NOT_FOUND, "User with email does not exist.")
            }
            _ => internal_error!("Failed to get user by email. Details: {}", e),
        })
    }

    async fn get_session_user(&self, session_user_id: &i64) -> Result<User> {
        sqlx::query_as!(
            User,
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
            session_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| internal_error!(e))
    }

    async fn create_user(&self, create_user: &CreateUser) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT
            INTO users (name, email, password)
            VALUES ($1, $2, $3)
            "#,
            &create_user.name,
            &create_user.email,
            &create_user.password,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| internal_error!(e))
    }

    async fn update_user(&self, update_user: &UpdateUser, session_user_id: &i64) -> Result<()> {
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
        .bind::<&i64>(session_user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::FORBIDDEN, "You are not permitted to modify this user.")
            }
            e => internal_error!(e),
        })
    }

    async fn delete_user(&self, session_user_id: &i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE
            FROM users
            WHERE id = $1
            "#,
            session_user_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                response_error!(StatusCode::FORBIDDEN, "You are not permitted to delete this user.")
            }
            e => internal_error!(e),
        })
    }
}
