use dioxus::prelude::*;
use dioxus_signals::{use_signal, Signal};
use shared::models::todo::Todo;

use crate::{api, components, handler::api_handler::ApiHandler, Message};

#[component]
pub(crate) fn TodoList(cx: Scope) -> Element {
    let error_handler: &Coroutine<crate::error::Error> =
        use_coroutine_handle(cx)?;
    let message_handler: &Coroutine<Message> = use_coroutine_handle(cx)?;
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let todo_list: Signal<Vec<Signal<Todo>>> = use_signal(cx, Vec::new);
    let todo_item_is_edited: Signal<Option<i64>> = use_signal(cx, || None);

    message_handler.send(Message("👋 Welcome back!".into()));

    let todo_list_future = use_future(cx, (), |_| {
        to_owned![api_handler, todo_list];
        async move {
            *todo_list.write() = api::todo::get_all_todos(&api_handler)
                .await
                .into_iter()
                .map(Signal::new)
                .collect();
        }
    });

    render! {
        match todo_list_future.value() {
            Some(_) => render! {
                div {
                    class: "space-y-2 dark:bg-zinc-800",
                    onclick: move |event| {
                        event.stop_propagation();
                        *todo_item_is_edited.write() = None;
                    },
                    h1 { class: "relative flex justify-left pt-8 pl-4 text-lg", "📥 Today" }
                    span {
                        class: "flex items-center",
                        span { class: "h-px flex-1 bg-white" }
                    }
                    ul {
                        for todo in todo_list.read().iter().filter(|todo| !todo.read().is_done) {
                            li {
                                components::todo::Todo { todo: *todo, is_edited: todo_item_is_edited }
                            }
                        }
                    }
                    h1 { class: "relative flex justify-left pt-8 pl-4 text-lg", "🗂️ Completed" }
                    span {
                        class: "flex items-center",
                        span { class: "h-px flex-1 bg-white" }
                    }
                    ul {
                        for todo in todo_list.read().iter().filter(|todo| todo.read().is_done) {
                            li {
                                components::todo::Todo { todo: *todo, is_edited: todo_item_is_edited }
                            }
                        }
                    }
                }
            },
            None => render! { div { "Loading todo list..." } },
        }
    }
}
