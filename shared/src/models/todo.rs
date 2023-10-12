use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub is_done: bool,
    pub owner: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct CreateTodo {
    pub title: String,
    pub description: String,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct UpdateTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_done: Option<bool>,
}
