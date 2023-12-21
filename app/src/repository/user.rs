use shared::models::user::{CreateUser, UpdateUser, User};

use super::error::{Operation, RepositoryError};
use crate::util::error_or::ErrorOr;

const RELATION: &str = "User";

/// SAFETY: never expose get_user_by_email to an endpoint directly
/// also never state that an email exists or does not exist.
/// One may state that both the email and password combination are invalid
/// without leaking additional information.
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn get_session_user(&self, session_user_id: &i64) -> ErrorOr<User>;

    async fn get_user_by_email(&self, email: &str) -> ErrorOr<User>;

    async fn create_user(&self, create_user: &CreateUser) -> ErrorOr<()>;

    async fn update_user(
        &self,
        update_user: &UpdateUser,
        session_user_id: &i64,
    ) -> ErrorOr<()>;

    async fn delete_user(&self, session_user_id: &i64) -> ErrorOr<()>;
}

pub struct PostgresUserRepository {
    pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn get_user_by_email(&self, email: &str) -> ErrorOr<User> {
        let db_response = sqlx::query_as!(
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
            sqlx::Error::RowNotFound => RepositoryError::NotFound {
                relation_name: RELATION.to_string(),
            },
            _ => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }

    async fn get_session_user(&self, session_user_id: &i64) -> ErrorOr<User> {
        let db_response = sqlx::query_as!(
            User,
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
            session_user_id
        )
        .fetch_one(&self.pool)
        .await?;

        db_response.into()
    }

    async fn create_user(&self, create_user: &CreateUser) -> ErrorOr<()> {
        let db_response = sqlx::query!(
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
        .map_err(Into::into)
        .map_err(RepositoryError::Internal)?;

        db_response.into()
    }

    // TODO apply same principle as in todo with optional type fields
    async fn update_user(
        &self,
        update_user: &UpdateUser,
        session_user_id: &i64,
    ) -> ErrorOr<()> {
        let db_response = sqlx::query(
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
            sqlx::Error::RowNotFound => RepositoryError::Forbidden {
                operation: Operation::Update,
                relation_name: RELATION.to_string(),
            },
            e => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }

    async fn delete_user(&self, session_user_id: &i64) -> ErrorOr<()> {
        let db_response = sqlx::query!(
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
            sqlx::Error::RowNotFound => RepositoryError::NotFound {
                relation_name: RELATION.to_string(),
            },
            e => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }
}
