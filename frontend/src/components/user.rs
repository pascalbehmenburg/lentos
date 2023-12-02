use crate::handler::api_handler::ApiHandler;
use dioxus::prelude::*;
use shared::models::user::User;

pub(crate) fn User(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();

    let user_future = use_future(cx, (), |_| {
        to_owned![api_handler];
        async move {
            let response = api_handler.get("/users").await;
            let user = response.json::<User>().await;
            user
        }
    });

    render! {
        match user_future.value() {
            Some(Ok(user)) => rsx! {
                div { "{user:?}" }
            },
            Some(Err(e)) => rsx! { div { "Error: {e}" } },
            None => rsx! { div { "Loading user..." } },
        },
    }
}
