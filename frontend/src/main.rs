#![allow(non_snake_case)]
use std::collections::{HashSet};

use app_dirs2::{app_root, AppDataType};
use async_std::stream::StreamExt;
use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::LogicalPosition;
use dioxus_desktop::tao::menu::{MenuBar, MenuItem};
use dioxus_desktop::{LogicalSize, WindowBuilder};
use dioxus_router::prelude::*;
use dioxus_signals::use_signal;
use reqwest::StatusCode;

mod api;
mod components;
mod error;
mod handler;

use components::sign_in::SignIn;
use components::sign_up::SignUp;
use components::todo_list::TodoList;
use components::user::User;
use handler::api_handler::ApiHandler;

use crate::components::popup::MessagePopup;

fn main() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::filter::*;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let fmt_layer = fmt::layer().with_target(true).pretty();

    let lib_filter_layer = Targets::new()
        .with_target("h2", LevelFilter::ERROR)
        .with_target("hyper", LevelFilter::ERROR)
        .with_default(LevelFilter::DEBUG);

    tracing_subscriber::registry()
        .with(lib_filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::install().unwrap();

    let mut menu_bar = MenuBar::new();

    // since it is uncommon on windows to have an "application menu"
    // we add a "window" menu to be more consistent across platforms with the
    // standard menu
    let mut window_menu = MenuBar::new();
    #[cfg(target_os = "macos")]
    {
        window_menu.add_native_item(MenuItem::EnterFullScreen);
        window_menu.add_native_item(MenuItem::Zoom);
        window_menu.add_native_item(MenuItem::Separator);
    }

    window_menu.add_native_item(MenuItem::Hide);

    #[cfg(target_os = "macos")]
    {
        window_menu.add_native_item(MenuItem::HideOthers);
        window_menu.add_native_item(MenuItem::ShowAll);
    }

    window_menu.add_native_item(MenuItem::Minimize);
    window_menu.add_native_item(MenuItem::CloseWindow);
    window_menu.add_native_item(MenuItem::Separator);
    window_menu.add_native_item(MenuItem::Quit);
    menu_bar.add_submenu("Window", true, window_menu);

    // since tao supports none of the below items on linux we should only add
    // them on macos/windows
    #[cfg(not(target_os = "linux"))]
    {
        let mut edit_menu = MenuBar::new();
        #[cfg(target_os = "macos")]
        {
            edit_menu.add_native_item(MenuItem::Undo);
            edit_menu.add_native_item(MenuItem::Redo);
            edit_menu.add_native_item(MenuItem::Separator);
        }

        edit_menu.add_native_item(MenuItem::Cut);
        edit_menu.add_native_item(MenuItem::Copy);
        edit_menu.add_native_item(MenuItem::Paste);

        #[cfg(target_os = "macos")]
        {
            edit_menu.add_native_item(MenuItem::Separator);
            edit_menu.add_native_item(MenuItem::SelectAll);
        }
        menu_bar.add_submenu("Edit", true, edit_menu);
    }

    let window = WindowBuilder::new()
        .with_title("Lentos")
        .with_always_on_top(true)
        .with_window_icon(None)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(390, 844))
        .with_position(LogicalPosition::new(1440 - 390, 0))
        .with_focused(false)
        .with_menu(menu_bar); // unsupported on ios / android

    let data_directory =
        app_root(AppDataType::UserConfig, &crate::handler::APP_INFO).unwrap();

    const INDEX_HTML: &str = r#"
        <!DOCTYPE html>
        <html class="h-full scroll-smooth" lang="en" dir="ltr">

        <head>
            <title>Lentos</title>
            <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <meta charset="UTF-8" />
            <link rel="stylesheet" href="tailwind.css">
        </head>

        <body
            class="font-sans antialiased \
                    bg-white text-zinc-950 \
                    dark:bg-zinc-900 dark:text-zinc-50">
            <div id="main"></div>
        </body>

        </html>
    "#;

    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_custom_index(INDEX_HTML.into())
            .with_data_directory(data_directory)
            .with_disable_context_menu(false)
            .with_background_color((0x18, 0x18, 0x1b, 100))
            .with_window(window),
    );
}

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(BaseLayer)]
    #[layout(PopupLayer)]
    #[layout(ErrorLayer)]
    #[route("/")]
    AuthCheck {},
    #[route("/signin")]
    SignIn {},
    #[route("/signup")]
    SignUp {},
    #[route("/user")]
    User {},
    #[route("/todolist")]
    TodoList {},
    #[end_layout]
    #[end_layout]
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn PageNotFound(cx: Scope, route: Vec<String>) -> Element {
    render! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}

#[component]
fn App(cx: Scope) -> Element {
    render! { Router::<Route> {} }
}

#[component]
fn BaseLayer(cx: Scope) -> Element {
    use_context_provider(cx, ApiHandler::new);

    render! {
        main { Outlet::<Route> {} }
    }
}

#[derive(Clone, Debug, PartialEq, derive_more::Display)]
enum Popup {
    #[display(fmt = "{}", _0)]
    Push(String),
    Pop(String),
}

#[component]
fn PopupLayer(cx: Scope) -> Element {
    let messages = use_signal(cx, HashSet::<String>::new);

    use_coroutine(cx, |mut receiver: UnboundedReceiver<Popup>| {
        to_owned![messages];
        async move {
            while let Some(command) = receiver.next().await {
                match command {
                    Popup::Push(msg) => {
                        messages.write().insert(msg);
                    }
                    Popup::Pop(msg) => {
                        messages.write().remove(&msg);
                    }
                }
            }
        }
    });

    render! {
        Outlet::<Route> {}
        ul { class: "fixed bottom-2 end-2 flex flex-col space-y-2 items-end",
            for msg in messages.read().iter() {
                li { class: "flex", MessagePopup { message: msg.clone() } }
            }
        }
    }
}

#[component]
fn ErrorLayer(cx: Scope) -> Element {
    let navigator = use_navigator(cx).clone();
    let popup_handler: &Coroutine<Popup> = use_coroutine_handle(cx).unwrap();

    use_coroutine(
        cx,
        |mut receiver: UnboundedReceiver<crate::error::Error>| {
            to_owned![navigator, popup_handler];
            async move {
                while let Some(error) = receiver.next().await {
                    tracing::error!("{:?}", error);
                    popup_handler.send(Popup::Push(error.1.to_string()));

                    let redirect = match error.0 {
                        reqwest::StatusCode::UNAUTHORIZED => {
                            navigator.push(Route::SignIn {})
                        }
                        _ => None,
                    }
                    .map(|e| {
                        tracing::error!("{:?}", e);
                        "Please log in first.".to_string()
                    });

                    if let Some(redirect_error_msg) = redirect {
                        popup_handler.send(Popup::Push(redirect_error_msg));
                    }
                }
            }
        },
    );

    render! { Outlet::<Route> {} }
}

#[component]
fn AuthCheck(cx: Scope) -> Element {
    let api_handler: &ApiHandler =
        use_context(cx).expect("Failed to receive api handler.");
    let navigator = use_navigator(cx);
    let message_handler: &Coroutine<Popup> = use_coroutine_handle(cx).unwrap();

    cx.spawn({
        to_owned![api_handler, navigator, message_handler];
        async move {
            let response = api_handler.get("/users").await;

            if response.status().is_success() {
                let user = response
                    .json::<shared::models::user::User>()
                    .await
                    .expect("Unable to read user data after login.");
                message_handler.send(Popup::Push(format!(
                    "👋 Welcome back, {}!",
                    user.name
                )));
                navigator.replace(Route::TodoList {});
            } else if response.status() == StatusCode::UNAUTHORIZED {
                navigator.replace(Route::SignIn {});
            } else {
                panic!("The backend server seems to be unresponsive.",);
            }
        }
    });

    render! {""}
}
