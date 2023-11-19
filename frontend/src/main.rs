#![allow(non_snake_case)]
use std::{collections::HashMap, sync::Arc, rc::Rc, path::Path, ops::Deref};

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use reqwest::cookie::Cookie;
use shared::models::user::{LoginUser, User, CreateUser};
use reqwest_cookie_store::CookieStoreMutex;
use dioxus_signals::Signal;
use reqwest::Client;

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

    let fmt_layer = fmt::layer().with_target(true).pretty();

    // default to error
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    // to filter output:
    // let filter = filter::Targets::new()
    //  .with_target("my_crate::uninteresting_module", LevelFilter::OFF);

    tracing_subscriber::registry()
        .with(filter_layer)
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
        // #[route("/")]
        // Home {},
        #[route("/")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/user")]
        UserPage {},
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

#[derive(Clone)]
pub struct ApiClient {
    inner: Rc<ApiClientInner>,
}

impl ApiClient {
    pub fn new(base_url: &'static str) -> Self{
        let cookie_store = get_cookie_store();
        let client = get_reqwest_client(cookie_store.clone());

        ApiClient {
            inner: Rc::new(ApiClientInner {
                client,
                cookie_store,
                base_url,
            }),
        }

    }
}

impl Deref for ApiClient {
    type Target = ApiClientInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ApiClientInner {
    pub(crate) client: Client,
    pub(crate) cookie_store: Arc<CookieStoreMutex>,
    pub(crate) base_url: &'static str,
}

impl Drop for ApiClientInner {
    fn drop(&mut self) {
        save_cookie_store(&self.cookie_store);
    }
}

fn log_cookie_store(cookie_store: &CookieStoreMutex) {
    tracing::debug!("Logging cookie store contents..");

    let store = cookie_store.lock().unwrap();

    for c in store.iter_any() {
        tracing::debug!("{:?}", c);
    }

    tracing::debug!("Logging cookie store contents done.");
}

fn load_cookie_store_from_file(cookie_path: &Path) -> reqwest_cookie_store::CookieStore {
    if cookie_path.exists() {
        let file = std::fs::File::open("cookies.json")
            .map(std::io::BufReader::new)
            .unwrap();
        reqwest_cookie_store::CookieStore::load_json(file).unwrap()
    } else {
        reqwest_cookie_store::CookieStore::new(None)
    }
}

fn get_cookie_store() -> Arc<CookieStoreMutex> {
    tracing::debug!("Loading cookie store..");

    let cookie_path = Path::new("cookies.json");

    let cookie_store = load_cookie_store_from_file(&cookie_path);
    let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);

    tracing::debug!("Cookie store loaded.");
    log_cookie_store(&cookie_store);

    cookie_store
}

fn save_cookie_store(cookie_store: &CookieStoreMutex) {
    tracing::debug!("Saving cookie store..");
    log_cookie_store(cookie_store);

    let mut writer = std::fs::File::create("cookies.json")
        .map(std::io::BufWriter::new)
        .unwrap();

    let store = cookie_store.lock().unwrap();
    store.save_json(&mut writer).unwrap();

    tracing::debug!("Cookie store saved.");
}

fn get_reqwest_client(cookie_store: Arc<CookieStoreMutex>) -> Client {
    let cert = std::fs::read("rootCA.pem").unwrap();
    let cert = reqwest::Certificate::from_pem(&cert).unwrap();

    reqwest::Client::builder()
        .cookie_store(true)
        .cookie_provider(cookie_store)
        .http2_prior_knowledge()
        .https_only(true)
        .use_rustls_tls()
        .add_root_certificate(cert)
        .build()
        .unwrap()
}

fn Base(cx: Scope) -> Element {
    use_context_provider(cx, || ApiClient::new("https://localhost:8443/api/v1"));

    render! {
        // The Outlet component will render child routes (In this case just the Home component) inside the Outlet component
        main { Outlet::<Route> {} }
    }
}

fn UserPage(cx: Scope) -> Element {
    let api_client: &ApiClient = use_context(cx).unwrap();

    let user_future = use_future(cx, (), |_| {
        to_owned![api_client];

        async move {
            let base_url = api_client.base_url;
            let client = &api_client.client;

            client
                .get(format!("{base_url}/users"))
                .send()
                .await
                .unwrap()
                .json::<User>()
                .await
        }
    });

    let test = use_future(cx, (), |_| {
        to_owned![api_client];

        async move {
            let base_url = api_client.base_url;
            let client = &api_client.client;

            client
                .get(format!("{base_url}/users"))
                .send()
                .await
                .unwrap()
                .json::<User>()
                .await
        }
    });

    render! {
        match user_future.value() {
            Some(Ok(user)) => rsx! {
                div { "{user:?}" }
            },
            Some(Err(e)) => rsx! { div { "Loading user failed {e:?}" } },
            None => rsx! { div { "Loading user..." } },
        },
        match test.value() {
            Some(Ok(user)) => rsx! {
                div { "{user:?}" }
            },
            Some(Err(e)) => rsx! { div { "Loading user failed {e:?}" } },
            None => rsx! { div { "Loading user..." } },
        }
    }
}

fn Register(cx: Scope) -> Element {
    let api_client: &Rc<ApiClient> = use_context(cx).unwrap();

    let registerHandler = move |createUser: CreateUser| {
        to_owned![api_client];

        cx.spawn(async move {
            tracing::debug!("Processing register event..");

            let register_request = &api_client.client
                .post("https://localhost:8443/api/v1/users/register")
                .json(&createUser)
                .send()
                .await
                .unwrap();

            if !register_request.status().is_success() {
                tracing::error!("Register failed. Server responded with: {:?}", register_request);
                return
            }

            tracing::debug!("Register event processed successfully. Server responded: {:?}", register_request);

            tracing::debug!("Logging in user in consequence of successful registration..");
            let login_user = LoginUser {
                email: createUser.email,
                password: createUser.password
            };

            // also persists the login
            login(login_user, &api_client).await;
        });
    };

    render! {
        form {
            onsubmit: move |evt| {
                tracing::debug!("Encountered event: {:?}", evt);
                evt.stop_propagation();
                let name = evt.values["username"].first().unwrap().to_string();
                let email = evt.values["email"].first().unwrap().to_string();
                let password = evt.values["password"].first().unwrap().to_string();
                registerHandler(CreateUser {
                    name,
                    email,
                    password,
                });
                use_navigator(cx).replace(Route::UserPage {});
            },
            class: "p-6 grid",
            label { class: "block mb-1", r#for: "username", "Username:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "username",
                id: "username",
                name: "username",
                placeholder: "Enter a username..",
                required: true
            }
            label { class: "block mb-1", r#for: "email", "Email:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "text",
                id: "email",
                name: "email",
                placeholder: "Enter an email..",
                required: true
            }
            label { class: "block mb-1", r#for: "password", "Password:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "password",
                id: "password",
                name: "password",
                placeholder: "Enter a password..",
                required: true
            }
            div { class: "flex flex-row justify-between items-center",
                button {
                    r#type: "submit",
                    class: "dark:bg-zinc-700
                            dark:hover:bg-zinc-600
                            bg-zinc-400
                            hover:bg-zinc-500
                            py-2
                            px-4
                            rounded",
                    "Sign Up"
                }
                Link { to: Route::Login {}, "Already have an account?" }
            }
        }
    }
}

async fn login(login_user: LoginUser, api_client: &ApiClient) {
    tracing::debug!("Trying to login with provided data..");

    let login_request = api_client.client
        .post("https://localhost:8443/api/v1/users/login")
        .json(&login_user)
        .send()
        .await
        .unwrap();

    if login_request.status().is_success() {
        tracing::error!("Login failed. Server responded with: {:?}", login_request);
        return
    }

    save_cookie_store(&api_client.cookie_store);
    tracing::debug!("Login processed.");
}

fn Login(cx: Scope) -> Element {
    let api_client: &ApiClient = use_context(cx).unwrap();

    let loginHandler = move |login_user: LoginUser| {
        to_owned![api_client];
        cx.spawn(async move {
            login(login_user, &api_client).await;
        });
    };

    render! {
        form {
            onsubmit: move |evt| {
                tracing::debug!("Encountered event: {:?}", evt);
                evt.stop_propagation();
                let email = evt.values["email"].first().unwrap().to_string();
                let password = evt.values["password"].first().unwrap().to_string();
                loginHandler(LoginUser { email, password });
                use_navigator(cx).replace(Route::UserPage {});
            },
            class: "p-6 grid",
            label { class: "block mb-1", r#for: "email", "Email:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "text",
                id: "email",
                name: "email",
                placeholder: "Enter your email..",
                required: true
            }
            label { class: "block mb-1", r#for: "password", "Password:" }
            input {
                class: "row dark:bg-zinc-800 mb-4 shadow appearance-none rounded py-3 px-4 leading-tight focus:outline-none focus:shadow-outline",
                r#type: "password",
                id: "password",
                name: "password",
                placeholder: "Enter your password..",
                required: true
            }
            div { class: "flex flex-row justify-between items-center",
                button {
                    r#type: "submit",
                    class: "dark:bg-zinc-700
                            dark:hover:bg-zinc-600
                            bg-zinc-400
                            hover:bg-zinc-500
                            py-2
                            px-4
                            rounded",
                    "Sign In"
                }
                Link { to: Route::Register {}, "Don't have an account?" }
            }
        }
    }
}