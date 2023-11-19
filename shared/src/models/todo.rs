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
pub struct FormUpdateTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_done: Option<String>,
}

impl From<FormUpdateTodo> for UpdateTodo {
    fn from(form_update_todo: FormUpdateTodo) -> Self {
        Self {
            id: form_update_todo.id,
            title: form_update_todo.title,
            description: form_update_todo.description,
            is_done: match form_update_todo.is_done {
                Some(is_done) => {
                    if is_done == "on" {
                        Some(true)
                    } else if is_done == "off" {
                        Some(false)
                    } else {
                        None
                    }
                }
                None => None,
            },
        }
    }
}
