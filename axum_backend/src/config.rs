use std::path::PathBuf;

use app_dirs2::{app_root, AppDataType, AppInfo};
use tokio::fs;
use toml_edit::Document;

use crate::internal_error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackendConfig {
    pub ip_address: String,
    pub http_port: usize,
    pub https_port: usize,
    pub cert_file_path: PathBuf,
    pub key_file_path: PathBuf,
    pub database_url: String,
    pub session_key: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            http_port: 8080,
            https_port: 8443,
            cert_file_path: PathBuf::from("cert.pem"),
            key_file_path: PathBuf::from("key.pem"),
            database_url: "postgres://postgres:postgres@localhost:17937/lentserver".to_string(),
            session_key: "gSFnDFaprJXK892HzwZU8KzWEQTBa5nh6QWjffv3bX9yZXr9DuZKqwYzq23haar4"
                .to_string(),
        }
    }
}

impl From<BackendConfig> for Document {
    fn from(config: BackendConfig) -> Self {
        toml_edit::ser::to_document(&config)
            .expect("Failed to serialize backend config to toml file.")
    }
}

impl BackendConfig {
    pub async fn load() -> crate::Result<BackendConfig> {
        const APP_INFO: AppInfo = AppInfo { name: "lentos_backend", author: "lentos" };

        let config_path = app_root(AppDataType::UserConfig, &APP_INFO)
            .map_err(|e| internal_error!("User config directory not found. Details: {}", e))?
            .join("Config.toml");

        if !config_path.exists() {
            let backend_config = BackendConfig::default();
            let toml_file = Document::from(backend_config.clone());

            fs::write(&config_path, toml_file.to_string()).await.map_err(|_| {
                internal_error!("Failed to write backend config to file {}.", config_path.display())
            })?;

            tracing::info!("Created new default config: {}", config_path.display());

            return backend_config.into();
        }

        let config = toml::from_str::<BackendConfig>(
            &std::fs::read_to_string(&config_path).map_err(|e| {
                internal_error!(
                    "Backend config has bad encoding. Ensure it is UTF-8. File: {}. Details: {}",
                    config_path.display(),
                    e
                )
            })?,
        )
        .map_err(|err| {
            internal_error!(
                "Failed to parse backend config. File: {}. Details: {}",
                config_path.display(),
                err
            )
        });

        tracing::info!("Loaded backend config: {}", config_path.display());

        config.into()
    }
}
