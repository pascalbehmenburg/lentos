use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml
pub(crate) mod api_handler;
pub(crate) mod cookie_handler;

pub(crate) const APP_INFO: AppInfo =
    AppInfo { name: "lentos", author: "lentos" };
