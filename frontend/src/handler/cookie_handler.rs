use std::{fmt::Display, path::PathBuf, sync::Arc};

use app_dirs2::{app_root, get_app_root, AppDataType};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

use crate::handler::APP_INFO;

// Wraps the cookie store into a handler which enforces default behaviours when
// using the CookieStore provided by reqwest_cookie_store::CookieStore
pub struct CookieHandler {
    cookie_store_path: PathBuf,
    cookie_store: Arc<CookieStoreMutex>,
}

impl CookieHandler {
    pub fn new() -> Self {
        let cookie_store_path = app_root(AppDataType::UserConfig, &APP_INFO)
            .expect("Could not find user config directory")
            .join("cookies.json");

        let cookie_store = if cookie_store_path.exists() {
            tracing::debug!(
                "Loading and parsing JSON file from {:?}",
                cookie_store_path
            );

            let file = std::fs::OpenOptions::new()
                .read(true)
                .open(&cookie_store_path)
                .map(std::io::BufReader::new)
                .expect("Failed to open cookie store file.");

            let cookie_store = CookieStore::load_json(file)
                .expect("Failed to parse cookie store file.");
            tracing::debug!("Loaded cookie store from file");
            cookie_store
        } else {
            tracing::debug!("Creating new cookie store");

            CookieStore::new(None)
        };

        tracing::debug!("Wrapping cookie store in Arc<CookieStoreMuter<>>");
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        Self { cookie_store_path, cookie_store }
    }

    pub fn save(&self) {
        tracing::debug!(
            "Saving cookie store to user dir with following contents: {:?} to {:?}",
            self.cookie_store.lock().unwrap(),
            self.cookie_store_path
        );

        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.cookie_store_path)
            .map(std::io::BufWriter::new)
            .unwrap();

        self.cookie_store.lock().unwrap().save_json(&mut writer).unwrap();
    }

    pub fn get_cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.cookie_store.clone()
    }
}
