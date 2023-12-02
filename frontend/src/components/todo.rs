use crate::api::*;
use crate::handler::api_handler::ApiHandler;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use shared::models::todo::Todo;

pub(crate) fn TodoList(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();

    let todo_list_future = use_future(cx, (), |_| {
        to_owned![api_handler];
        async move { todo::get_all_todos(&api_handler).await }
    });

    render! {
        match todo_list_future.value() {
            Some(todo_list) => rsx! {
                ul {
                    todo_list.iter().map(|todo| {
                        rsx! {
                            li {
                                Todo { ..todo.clone() }
                            }
                        }
                    })
                }
            },
            None => rsx! { div { "Loading todo list..." } },
        },
    }
}

pub(crate) fn Todo(cx: Scope<Todo>) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let navigator = use_navigator(cx);

    render! {
        div {
            class: "flex max-w-xs items-center space-x-2 rounded-lg dark:bg-zinc-800 p-4 shadow-lg",
            form {
                method: "PUT",
                button {
                    class: "flex h-6 w-6 cursor-pointer items-center justify-center rounded border dark:bg-zinc-700 dark:hover:bg-zinc-600 bg-zinc-400 hover:bg-zinc-500",
                    r#type: "submit",
                    name: "is_done",
                    id: "is_done"
                }
                div {
                    class: "flex flex-col",
                    div {
                        p { "True" }
                        p { "False" }
                    }
                }
            }
        }
    }
}
