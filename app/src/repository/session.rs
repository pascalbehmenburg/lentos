use actix_session::storage::{
    LoadError, SaveError, SessionKey, SessionStore, UpdateError,
};
use actix_web::cookie::time::Duration;
use anyhow::Context;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This struct is used to represent a actix session in a sql database
/// in this case postgresql using the following sql schema:
/// ```sql
/// CREATE TABLE sessions (
///   key char(64) NOT NULL UNIQUE,
///   state jsonb NOT NULL,
///   CONSTRAINT sessions_pkey PRIMARY KEY (key)
/// );
/// ```
/// also one should use this index to increase query performance:
/// CREATE INDEX session_key_index ON sessions USING hash (key);
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub key: String,
    pub state: sqlx::types::Json<serde_json::Value>,
}

impl Session {
    pub fn new(
        key: String,
        state: sqlx::types::Json<serde_json::Value>,
    ) -> Result<Self, &'static str> {
        if key.len() > 64 {
            return Err("Session key cannot be longer than 64 bytes");
        }
        Ok(Self { key, state })
    }
}

pub type SessionState = HashMap<String, String>;

#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync + 'static {
    async fn db_load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<Session>, sqlx::Error>;

    async fn db_save(
        &self,
        session_key: &SessionKey,
        session_state: &serde_json::Value,
    ) -> Result<(), sqlx::Error>;

    async fn db_update(
        &self,
        session_key: &SessionKey,
        session_state: &serde_json::Value,
    ) -> Result<(), sqlx::Error>;

    async fn db_delete(
        &self,
        session_key: &SessionKey,
    ) -> Result<(), sqlx::Error>;

    async fn generate_session_key() -> SessionKey {
        let value = std::iter::repeat(())
            .map(|()| OsRng.sample(Alphanumeric))
            .take(64)
            .collect::<Vec<_>>();
        String::from_utf8(value).unwrap().try_into().unwrap()
    }
}

#[derive(Clone)]
pub struct PostgresSessionRepository {
    pool: sqlx::PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn db_load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<Session>, sqlx::Error> {
        let db_response = sqlx::query_as!(
            Session,
            r#"
      SELECT key, state
      FROM sessions
      WHERE key = $1
      "#,
            session_key.as_ref()
        )
        .fetch_optional(&self.pool)
        .await;

        return db_response;
    }

    async fn db_save(
        &self,
        session_key: &SessionKey,
        session_state: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let db_response = sqlx::query!(
            r#"
      INSERT
      INTO sessions (key, state)
      VALUES ($1, $2)
      "#,
            session_key.as_ref(),
            session_state
        )
        .execute(&self.pool)
        .await
        .map(|_| ());

        return db_response;
    }

    async fn db_update(
        &self,
        session_key: &SessionKey,
        session_state: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let db_response = sqlx::query!(
            r#"
      UPDATE sessions
      SET state = $1
      WHERE key = $2
      "#,
            session_state,
            session_key.as_ref()
        )
        .execute(&self.pool)
        .await
        .map(|_| ());

        return db_response;
    }

    async fn db_delete(
        &self,
        session_key: &SessionKey,
    ) -> Result<(), sqlx::Error> {
        let db_response = sqlx::query!(
            r#"
      DELETE
      FROM sessions
      WHERE key = $1
      "#,
            session_key.as_ref()
        )
        .execute(&self.pool)
        .await
        .map(|_| ());

        return db_response;
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for PostgresSessionRepository {
    async fn load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<SessionState>, LoadError> {
        // try to load session from db
        let db_response = self
            .db_load(session_key)
            .await
            .map_err(Into::into)
            .map_err(LoadError::Other);

        // transforms Session to SessionState while keeping the result
        let session_state: Result<Option<SessionState>, LoadError> =
            db_response.and_then(|db_session| {
                db_session
                    .map(|s| {
                        let session_state_val = s.state.0;

                        // deserialize
                        serde_json::from_value::<SessionState>(
                            session_state_val,
                        )
                        .map_err(Into::into)
                        .map_err(LoadError::Deserialization)
                    })
                    .transpose()
            });

        return session_state;
    }

    async fn save(
        &self,
        session_state: SessionState,
        _ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let session_key: SessionKey = Self::generate_session_key().await;

        let session_state = serde_json::to_value(session_state)
            .map_err(Into::into)
            .map_err(SaveError::Serialization)?;

        self.db_save(&session_key, &session_state)
            .await
            .map_err(Into::into)
            .map_err(SaveError::Other)?;

        return Ok(session_key);
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        _ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let session_state = serde_json::to_value(session_state)
            .map_err(Into::into)
            .map_err(UpdateError::Serialization)?;

        self.db_update(&session_key, &session_state)
            .await
            .map_err(Into::into)
            .map_err(UpdateError::Other)?;

        return Ok(session_key);
    }

    async fn update_ttl(
        &self,
        _session_key: &SessionKey,
        _ttl: &Duration,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn delete(
        &self,
        session_key: &SessionKey,
    ) -> Result<(), anyhow::Error> {
        let db_response = self
      .db_delete(session_key)
      .await
      .map_err(anyhow::Error::from)
      .context(
        "Some psql error occurred when trying to delete session from db.",
      );

        return db_response;
    }
}
