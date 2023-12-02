#![allow(non_snake_case)]
use std::ops::Deref;
use app_dirs2::{app_root, AppDataType};

use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::{LogicalPosition, PhysicalPosition};
use dioxus_desktop::{LogicalSize, PhysicalSize, WindowBuilder};
use dioxus_desktop::tao::monitor;
use dioxus_desktop::tao::monitor::MonitorHandle;
use dioxus_desktop::tao::window::Fullscreen;
use dioxus_desktop::tao::window::Fullscreen::Borderless;
use dioxus_router::prelude::*;
use Fullscreen::Exclusive;
use reqwest::StatusCode;
use tracing_subscriber::filter::Targets;

mod handler;
mod components;
mod api;

use handler::api_handler::ApiHandler;
use components::sign_up::SignUp;
use components::sign_in::SignIn;
use components::user::User;
use components::todo::TodoList;

pub static INDEX_HTML: &str = r#"
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

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{prelude::*, EnvFilter};
    use tracing_subscriber::fmt;
    use tracing_subscriber::filter::*;

    let fmt_layer = fmt::layer().with_target(true).pretty();

    let lib_filter_layer = Targets::new()
        .with_target("h2", LevelFilter::ERROR)
        .with_target("hyper", LevelFilter::ERROR)
        .with_default(LevelFilter::DEBUG);;

    tracing_subscriber::registry()
        .with(lib_filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

fn main() {
    install_tracing();
    color_eyre::install().unwrap();

    let data_directory = app_root(AppDataType::UserConfig, &crate::handler::APP_INFO)
        .unwrap();

    let window = WindowBuilder::new()
        .with_title("Lentos")
        .with_always_on_top(true)
        .with_window_icon(None)
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(600, 1080))
        .with_position(LogicalPosition::new(1920-600, 0))
        .with_focused(false); // unsopported on ios / android

    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_custom_index(INDEX_HTML.to_string())
            .with_data_directory(data_directory)
            .with_disable_context_menu(true)
            .with_background_color((0x18, 0x18, 0x1b, 100))
            .with_window(window),
    );
}

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(BaseLayout)]
        #[route("/")]
        Redirection {},
        #[route("/signin")]
        SignIn {},
        #[route("/signup")]
        SignUp {},
        #[route("/user")]
        User {},
        #[route("/todo")]
        TodoList {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}

#[inline_props]
fn PageNotFound(cx: Scope, route: Vec<String>) -> Element {
    render! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}

fn App(cx: Scope) -> Element {
    render! { Router::<Route> {} }
}

fn BaseLayout(cx: Scope) -> Element {
    use_context_provider(cx, || ApiHandler::new());

    render! {
        // The Outlet component will render child routes (In this case just the Home component) inside the Outlet component
        main { Outlet::<Route> {} }
    }
}

fn Redirection(cx: Scope) -> Element {
    let api_handler: &ApiHandler = use_context(cx).unwrap();
    let navigator = use_navigator(cx);

    cx.spawn({
        to_owned![api_handler, navigator];
        async move {
            let response = api_handler.get("/users").await;

            if response.status().is_success() {
                navigator.replace(Route::TodoList {});
            } else if response.status() == StatusCode::UNAUTHORIZED {
                navigator.replace(Route::SignIn {});
            } else {
                panic!("The backend server seems to be unresponsive: {:?}", response);
            }
        }
    });

    render! {
        ""
    }
}