use crate::api::*;
use crate::components::check_box::CheckBox;
use crate::handler::api_handler::ApiHandler;
use dioxus::prelude::*;
use dioxus_signals::Signal;
use shared::models::todo::{Todo, UpdateTodo};

#[component]
pub(crate) fn Todo(
    cx: Scope,
    todo: Signal<Todo>,
    is_edited: Signal<Option<i64>>,
) -> Element {
    let error_handler: &Coroutine<crate::error::Error> =
        use_coroutine_handle(cx)?;
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let todo_reader = todo.read().clone();
    let is_edited_reader = match *is_edited.read() {
        Some(id) => id == todo_reader.id,
        None => false,
    };

    let line_through_css_class =
        if todo_reader.is_done { "line-through" } else { "" };

    let is_done_update_handler = move |update_todo: UpdateTodo| {
        to_owned![api_handler, todo, error_handler];

        cx.spawn(async move {
            let status_code =
                todo::update_todo(&api_handler, update_todo).await;
            if status_code.is_success() {
                let is_done = todo.peek().is_done;
                todo.write().is_done = !is_done;
            } else {
                error_handler.send(crate::error::Error(
                    status_code,
                    "Failed to update todo.".into(),
                ));
            }
        });
    };

    let update_todo_handler = move |update_todo: UpdateTodo| {
        to_owned![api_handler, is_edited, error_handler];

        cx.spawn(async move {
            let status_code =
                todo::update_todo(&api_handler, update_todo).await;
            if status_code.is_success() {
                *is_edited.write() = None;
            } else {
                error_handler.send(crate::error::Error(
                    status_code,
                    "Failed to update todo.".into(),
                ));
            }
        });
    };

    render! {
        div { class: "items-left flex space-x-2 px-4 py-3 dark:bg-zinc-800",
            button {
                onclick: move |event| {
                    event.stop_propagation();
                    let todo_reader = todo.read();
                    is_done_update_handler(UpdateTodo {
                        id: todo_reader.id,
                        title: None,
                        description: None,
                        is_done: Some(!todo_reader.is_done),
                    });
                },
                r#type: "checkbox",
                name: "is_done",
                class: "mt-1 flex cursor-pointer h-5 w-5 flex-col items-center rounded border dark:border-zinc-500 dark:hover:bg-zinc-700",
                CheckBox { checked: todo_reader.is_done }
            }
            div { class: "flex flex-col w-full",
                if !is_edited_reader {
                    render! {
                        div {
                            class: "cursor-pointer",
                            onclick: move |event| {
                                event.stop_propagation();
                                *is_edited.write() = Some(todo_reader.id);
                            },
                            p {
                                class: "text-lg {line_through_css_class}",
                                "{todo_reader.title}"
                            }
                            p {
                                class: "text-sm dark:text-zinc-400",
                                "{todo_reader.description}"
                            }
                        }
                    }
                } else {
                    render! {
                        form {
                            onsubmit: move |event| {
                                event.stop_propagation();
                                let todo_reader = todo.read();
                                update_todo_handler(UpdateTodo {
                                    id: todo_reader.id,
                                    title: Some(todo_reader.title.clone()),
                                    description: Some(todo_reader.description.clone()),
                                    is_done: None,
                                });
                            },
                            input {
                                r#type: "text",
                                name: "title",
                                placeholder: "Title",
                                value: "{todo_reader.title}",
                                class: "w-full overflow-hidden border-b border-transparent bg-transparent text-lg focus:outline-none dark:text-zinc-50 dark:placeholder:text-zinc-50 focus:dark:border-zinc-500",
                                onmounted: move |event| async move {
                                    event.inner().set_focus(true).await.expect("Failed to set focus of Todo component.");
                                },
                                oninput: move |evt| {
                                    todo.write().title = evt.value.clone();
                                },
                            }
                            input {
                                r#type: "text",
                                name: "description",
                                placeholder: "Description",
                                value: "{todo_reader.description}",
                                class: "w-full overflow-hidden border-b border-transparent bg-transparent text-sm focus:outline-none dark:text-zinc-400 dark:placeholder:text-zinc-400 focus:dark:border-zinc-500",
                                oninput: move |evt| {
                                    todo.write().description = evt.value.clone();
                                },
                            }
                            div {
                                class: "flex justify-end space-x-2 mt-2",
                                button {
                                    class: "rounded bg-zinc-300 px-3 py-1 text-zinc-950 hover:bg-gray-200",
                                    r#type: "button",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        *is_edited.write() = None;
                                    },
                                    "Cancel"
                                }
                                button {
                                    class: "rounded bg-sky-600 px-3 py-1 text-white hover:bg-sky-500",
                                    r#type: "submit",
                                    "Save"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
