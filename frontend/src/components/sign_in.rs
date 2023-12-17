use crate::handler::api_handler::ApiHandler;
use crate::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_router::prelude::Link;
use shared::models::user::SignInUser;

#[component]
pub(crate) fn SignIn(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let navigator = use_navigator(cx);

    let sign_in_handler = move |sign_in_user: SignInUser| {
        to_owned![api_handler, navigator];

        cx.spawn(async move {
            crate::api::auth::sign_in(&api_handler, sign_in_user).await;
            navigator.replace(Route::TodoList {});
        });
    };

    render! {
        form {
            onsubmit: move |event| {
                tracing::debug!("Encountered event: {:?}", event);
                event.stop_propagation();
                let email = event.values["email"].first().unwrap().to_string();
                let password = event.values["password"].first().unwrap().to_string();
                let all_values = event.values.clone();
                tracing::debug!("{:?}", all_values);
                sign_in_handler(SignInUser { email, password });
            },
            class: "p-6 grid",
            label { class: "block mb-1", r#for: "email", "Email:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "text",
                id: "email",
                name: "email",
                placeholder: "Enter your email...",
                required: true
            }
            label { class: "block mb-1", r#for: "password", "Password:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "password",
                id: "password",
                name: "password",
                placeholder: "Enter your password...",
                required: true
            }
            div { class: "flex flex-row justify-between items-center",
                button {
                    r#type: "submit",
                    class: "dark:bg-zinc-700
                            dark:hover:bg-zinc-600
                            bg-zinc-400
                            hover:bg-zinc-500
                            py-2
                            px-4
                            rounded",
                    "Sign In"
                }
                Link { to: Route::SignUp {}, "Don't have an account?" }
            }
        }
    }
}
