use shared::models::todo::{CreateTodo, Todo, UpdateTodo};

use super::error::{Operation, RepositoryError};
use crate::util::error_or::ErrorOr;

const RELATION: &str = "Todo";

#[async_trait::async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    async fn get_todos(&self, session_user_id: &i64) -> ErrorOr<Vec<Todo>>;

    async fn get_todo(
        &self,
        todo_id: &i64,
        session_user_id: &i64,
    ) -> ErrorOr<Todo>;

    async fn create_todo(
        &self,
        create_todo: &CreateTodo,
        session_user_id: &i64,
    ) -> ErrorOr<()>;

    async fn update_todo(
        &self,
        update_todo: &UpdateTodo,
        session_user_id: &i64,
    ) -> ErrorOr<Todo>;

    async fn delete_todo(&self, id: &i64, session_user_id: &i64)
    -> ErrorOr<()>;
}

pub struct PostgresTodoRepository {
    pool: sqlx::PgPool,
}

impl PostgresTodoRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TodoRepository for PostgresTodoRepository {
    async fn get_todos(&self, session_user_id: &i64) -> ErrorOr<Vec<Todo>> {
        let db_response = sqlx::query_as!(
            Todo,
            r#"
            SELECT *
            FROM todos
            WHERE owner = $1
            ORDER BY id"#,
            session_user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound {
                relation_name: RELATION.to_string(),
            },
            _ => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }

    async fn get_todo(
        &self,
        todo_id: &i64,
        session_user_id: &i64,
    ) -> ErrorOr<Todo> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            SELECT *
            FROM todos
            WHERE id = $1
            "#,
            todo_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound {
                relation_name: RELATION.to_string(),
            },
            e => RepositoryError::Internal(e.into()),
        })?;

        // check if session user that made the request
        // is the actual owner of the todo that was requested
        let todo = if todo.owner == *session_user_id {
            Ok(todo)
        } else {
            Err(RepositoryError::Forbidden {
                operation: Operation::Receive,
                relation_name: RELATION.to_string(),
            })
        }?;

        todo.into()
    }

    async fn create_todo(
        &self,
        create_todo: &CreateTodo,
        session_user_id: &i64,
    ) -> ErrorOr<()> {
        let db_response = sqlx::query!(
            r#"
            INSERT
            INTO todos (title, description, owner)
            VALUES ($1, $2, $3)
            "#,
            &create_todo.title,
            &create_todo.description,
            session_user_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
        .map_err(RepositoryError::Internal)?;

        db_response.into()
    }

    async fn update_todo(
        &self,
        update_todo: &UpdateTodo,
        session_user_id: &i64,
    ) -> ErrorOr<Todo> {
        let db_response = sqlx::query_as::<_, Todo>(
            r#"
            UPDATE todos
            SET
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                is_done = COALESCE($3, is_done),
                updated_at = NOW()
            WHERE id = $4 and owner = $5
            RETURNING *
            "#,
        )
        .bind::<&Option<String>>(&update_todo.title)
        .bind::<&Option<String>>(&update_todo.description)
        .bind::<&Option<bool>>(&update_todo.is_done)
        .bind::<&i64>(&update_todo.id)
        .bind::<&i64>(session_user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::Forbidden {
                operation: Operation::Update,
                relation_name: RELATION.to_string(),
            },
            e => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }

    async fn delete_todo(
        &self,
        todo_id: &i64,
        session_user_id: &i64,
    ) -> ErrorOr<()> {
        let db_response = sqlx::query!(
            r#"
            DELETE
            FROM todos
            WHERE id = $1 and owner = $2
            "#,
            todo_id,
            session_user_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::Forbidden {
                operation: Operation::Delete,
                relation_name: RELATION.to_string(),
            },
            _ => RepositoryError::Internal(e.into()),
        })?;

        db_response.into()
    }
}
