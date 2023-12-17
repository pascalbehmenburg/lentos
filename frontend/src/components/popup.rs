use dioxus::prelude::*;

#[component]
pub fn Popup(cx: Scope, text: String) -> Element {
    render! {
        aside { class: "fixed bottom-4 end-4 z-50 flex items-center justify-center gap-4 rounded-lg bg-black px-5 py-3 text-white",
            a {
                href: "/new-thing",
                target: "_blank",
                rel: "noreferrer",
                class: "text-sm font-medium hover:opacity-75",
                "\n    {text}\n  "
            }
            button { class: "rounded bg-white/20 p-1 hover:bg-white/10",
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
