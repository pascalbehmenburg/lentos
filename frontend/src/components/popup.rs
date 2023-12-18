use dioxus::prelude::*;

use crate::Popup;

#[component]
pub fn MessagePopup(cx: Scope, message: String) -> Element {
    let message_handler: &Coroutine<Popup> = use_coroutine_handle(cx).unwrap();

    use_on_create(cx, || {
        to_owned![message, message_handler];
        async move {
            async_std::task::sleep(std::time::Duration::from_secs(6)).await;
            message_handler.send(Popup::Pop(message.to_string()));
        }
    });

    render! {
        aside { class: "flex items-center justify-center gap-4 rounded-lg dark:bg-zinc-800 px-5 py-3 border dark:border-zinc-500",
            span { class: "text-sm font-medium", "\n    {message}\n  " }
            button {
                class: "rounded p-1 dark:bg-zinc-700 dark:hover:bg-zinc-600 bg-zinc-400 hover:bg-zinc-500",
                onclick: move |_| {
                    message_handler.send(Popup::Pop(message.to_string()));
                },
                span { class: "sr-only", "Close" }
                svg {
                    fill: "currentColor",
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 20 20",
                    class: "h-4 w-4",
                    path {
                        d: "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z",
                        clip_rule: "evenodd",
                        fill_rule: "evenodd"
                    }
                }
            }
        }
    }
}
