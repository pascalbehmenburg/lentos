use crate::handler::cookie_handler::CookieHandler;
use reqwest::Client;
use std::ops::Deref;
use std::rc::Rc;

const BASE_URL: &str = "https://localhost:8443/api/v1";

#[derive(Clone)]
pub struct ApiHandler {
    api_client_wrapper: Rc<ApiClientWrapper>,
}

impl ApiHandler {
    pub fn new(base_url: &'static str) -> Self {
        let cookie_store_handler = CookieHandler::new();
        let cookie_store = cookie_store_handler.get_cookie_store();

        let cert = std::fs::read("rootCA.pem").unwrap();
        let cert = reqwest::Certificate::from_pem(&cert).unwrap();

        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store)
            .http2_prior_knowledge()
            .https_only(true)
            .use_rustls_tls()
            .add_root_certificate(cert)
            .build()
            .unwrap();

        ApiHandler {
            api_client_wrapper: Rc::new(ApiClientWrapper {
                client,
                cookie_store: cookie_store_handler,
                base_url,
            }),
        }
    }
}

impl Deref for ApiHandler {
    type Target = ApiClientWrapper;

    fn deref(&self) -> &Self::Target {
        &self.api_client_wrapper
    }
}

pub struct ApiClientWrapper {
    pub(crate) client: Client,
    pub(crate) cookie_store: CookieHandler,
    pub(crate) base_url: &'static str,
}

impl Drop for ApiClientWrapper {
    fn drop(&mut self) {
        self.cookie_store.save();
    }
}
