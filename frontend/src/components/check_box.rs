use dioxus::prelude::*;

#[component]
pub fn CheckBox(cx: Scope, checked: bool) -> Element {
    if *checked {
        render! {
            svg {
                width: "32",
                view_box: "0 -960 960 960",
                height: "32",
                xmlns: "http://www.w3.org/2000/svg",
                class: "fill-current opacity-100",
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
