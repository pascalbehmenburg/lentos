use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml
pub(crate) mod api_client;
pub(crate) mod cookie_handler;

const APP_INFO: AppInfo = AppInfo { name: "lentos", author: "lentos" };
