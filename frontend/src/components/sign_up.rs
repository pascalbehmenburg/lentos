use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_router::prelude::Link;
use shared::models::user::CreateUser;

use crate::api::auth::sign_up;
use crate::handler::api_handler::ApiHandler;
use crate::Route;

#[component]
pub(crate) fn SignUp(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let navigator = use_navigator(cx);

    let sign_up_handler = move |createUser: CreateUser| {
        to_owned![api_handler, navigator];

        cx.spawn(async move {
            sign_up(&api_handler, createUser).await;
            navigator.replace(Route::TodoList {});
        });
    };

    render! {
        form {
            onsubmit: move |event| {
                tracing::debug!("Encountered event: {:?}", event);
                event.stop_propagation();
                let name = event.values["username"].first().unwrap().to_string();
                let email = event.values["email"].first().unwrap().to_string();
                let password = event.values["password"].first().unwrap().to_string();
                sign_up_handler(CreateUser {
                    name,
                    email,
                    password,
                });
            },
            class: "p-6 grid",
            label { class: "block mb-1", r#for: "username", "Username:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "username",
                id: "username",
                name: "username",
                placeholder: "Enter a username...",
                required: true
            }
            label { class: "block mb-1", r#for: "email", "Email:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "text",
                id: "email",
                name: "email",
                placeholder: "Enter an email...",
                required: true
            }
            label { class: "block mb-1", r#for: "password", "Password:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "password",
                id: "password",
                name: "password",
                placeholder: "Enter a password...",
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
                    "Sign Up"
                }
                Link { to: Route::SignIn {}, "Already have an account?" }
            }
        }
    }
}
