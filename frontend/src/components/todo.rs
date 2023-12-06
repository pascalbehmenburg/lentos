use dioxus::html::input_data::keyboard_types;
use crate::api::*;
use crate::handler::api_handler::ApiHandler;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use shared::models::todo::{Todo, UpdateTodo};
use shared::models::user::CreateUser;
use crate::api::auth::sign_up;
use crate::Route;

pub(crate) fn TodoList(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();

    let todo_list_future = use_future(cx, (), |_| {
        to_owned![api_handler];
        async move { todo::get_all_todos(&api_handler).await }
    });

    render! {
        match todo_list_future.value() {
            Some(todo_list) => render! {
                ul {
                    todo_list.iter().map(|todo| {
                        render! {
                            li {
                                Todo { ..todo.clone() }
                            }
                        }
                    })
                }
            },
            None => render! { div { "Loading todo list..." } },
        },
    }
}

#[derive(PartialEq, Props)]
struct CheckMarkProps {
    checked: bool,
}
fn CheckMark(cx: Scope<CheckMarkProps>) -> Element {
    if cx.props.checked {
        render! {
            svg {
                width: "32",
                view_box: "0 -960 960 960",
                height: "32",
                xmlns: "http://www.w3.org/2000/svg",
                class: "fill-current opacity-100 hover:opacity-0",
                path { d: "M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z" }
            }
        }
    } else {
        render! {
            svg {
                width: "32",
                view_box: "0 -960 960 960",
                height: "32",
                xmlns: "http://www.w3.org/2000/svg",
                class: "fill-current opacity-0 hover:opacity-100",
                path { d: "M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z" }
            }
        }
    }
}

pub(crate) fn Todo(cx: Scope<Todo>) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let todo = use_ref(cx, ||  cx.props.clone());

    let is_edited = use_state(cx, || false);
    let line_through_css_class = if todo.read().is_done { "line-through" } else { "" };

    let is_done_update_handler = move |updateTodo: UpdateTodo| {
        to_owned![api_handler, todo];

        cx.spawn(async move {
            let status_code = todo::update_todo(&api_handler, updateTodo).await;
            if status_code.is_success() {
                todo.with_mut(|todo| {
                    todo.is_done = !todo.is_done;
                });
                todo.needs_update();
            } else {
                // todo tell user that update failed
            }
        });
    };

    let update_todo_handler = move |updateTodo: UpdateTodo| {
        to_owned![api_handler, is_edited];

        cx.spawn(async move {
            let status_code = todo::update_todo(&api_handler, updateTodo).await;
            if status_code.is_success() {
                is_edited.set(false);
                is_edited.needs_update();
            } else {
                // todo tell user that update failed
            }
        });
    };

    render! {
        div {
            class: "items-left flex space-x-2 px-4 py-3 dark:bg-zinc-800",
            button {
                onclick: move |event| {
                    tracing::debug!("Encountered event: {:?}", event);
                    event.stop_propagation();

                    is_done_update_handler(UpdateTodo {
                        id: cx.props.id,
                        title: None,
                        description: None,
                        is_done: Some(!todo.read().is_done)
                    });
                },
                r#type: "checkbox",
                name: "is_done",
                class: "mt-1 flex cursor-pointer h-5 w-5 flex-col items-center rounded border dark:border-zinc-500 dark:hover:bg-zinc-700",
                    CheckMark { checked: todo.read().is_done }
            }
            div {
                class: "flex flex-col {line_through_css_class} w-full",
                if !is_edited {
                    render! {
                        div {
                            class: "cursor-pointer",
                            onclick: move |event| {
                                event.stop_propagation();
                                is_edited.set(!is_edited.get());
                            },
                            p {
                                class: "text-lg",
                                "{todo.read().title}"
                            }
                            p {
                                class: "text-sm dark:text-zinc-400",
                                "{todo.read().description}"
                            }
                        }
                    }
                } else {
                    render! {
                        div {
                            class: "",
                            input {
                                r#type: "text",
                                name: "title",
                                placeholder: "Title",
                                value: "{todo.read().title}",
                                class: "w-full overflow-hidden border-b border-transparent bg-transparent text-lg focus:outline-none dark:text-zinc-50 dark:placeholder:text-zinc-50 focus:dark:border-zinc-500",
                                autofocus: true,
                                onmounted: move |event| {
                                    event.inner().set_focus(true);
                                },
                                oninput: move |evt| todo.with_mut(|todo| todo.title = evt.value.clone()),
                                onkeypress: move |evt| {
                                    if evt.key() == keyboard_types::Key::Enter {
                                        update_todo_handler(UpdateTodo {
                                            id: cx.props.id,
                                            ..todo.read().clone().into()
                                        });
                                    }
                                },
                            }
                            input {
                                r#type: "text",
                                name: "description",
                                placeholder: "Description",
                                value: "{todo.read().description}",
                                class: "w-full overflow-hidden border-b border-transparent bg-transparent text-sm focus:outline-none dark:text-zinc-400 dark:placeholder:text-zinc-400 focus:dark:border-zinc-500",
                                oninput: move |evt| todo.with_mut(|todo| todo.description = evt.value.clone()),
                            }
                            div {
                                class: "flex justify-end space-x-2 mt-2",
                                button {
                                    class: "rounded bg-zinc-300 px-3 py-1 text-zinc-950 hover:bg-gray-200",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        is_edited.set(!is_edited.get());
                                    },
                                    "Cancel"
                                }
                                button {
                                    class: "rounded bg-sky-600 px-3 py-1 text-white hover:bg-sky-500",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        update_todo_handler(UpdateTodo {
                                            id: cx.props.id,
                                            ..todo.read().clone().into()
                                        });
                                    },
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
