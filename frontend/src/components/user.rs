use crate::handler::api_client::ApiHandler;
use dioxus::prelude::*;
use shared::models::user::User;

pub(crate) fn User(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();

    let user_future = use_future(cx, (), |_| {
        to_owned![api_handler];
        async move {
            let base_url = api_handler.base_url;

            api_handler
                .client
                .get(format!("{base_url}/users"))
                .send()
                .await
                .unwrap()
                .json::<User>()
                .await
        }
    });

    render! {
        match user_future.value() {
            Some(Ok(user)) => rsx! {
                div { "{user:?}" }
            },
            Some(Err(e)) => rsx! { div { "Loading user failed {e:?}" } },
            None => rsx! { div { "Loading user..." } },
        },
    }
}
