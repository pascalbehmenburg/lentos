use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::handler::api_handler::ApiHandler;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserView {
    pub id: i64,
    pub anonymous: bool,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[component]
pub(crate) fn User(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();

    let user_future = use_future(cx, (), |_| {
        to_owned![api_handler];
        async move {
            let response = api_handler.get("/users").await;
            response.json::<UserView>().await
        }
    });

    render! {
        match user_future.value() {
            Some(Ok(user)) => rsx! {
                div { "{user:?}" }
            },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading user..." } },
        }
    }
}
