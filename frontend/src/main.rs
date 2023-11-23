#![allow(non_snake_case)]
use std::ops::Deref;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use tracing_subscriber::filter::Targets;

mod handler;
mod components;
mod api;

use handler::api_client::ApiHandler;
use components::sign_up::SignUp;
use components::sign_in::SignIn;
use components::user::User;

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

    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_custom_index(INDEX_HTML.to_string()),
    );
}

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(Base)]
        #[route("/")]
        SignIn {},
        #[route("/register")]
        SignUp {},
        #[route("/user")]
        User {},
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

fn Base(cx: Scope) -> Element {
    use_context_provider(cx, || ApiHandler::new("https://localhost:8443/api/v1"));

    render! {
        // The Outlet component will render child routes (In this case just the Home component) inside the Outlet component
        main { Outlet::<Route> {} }
    }
}