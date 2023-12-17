use std::borrow::Cow;

use reqwest::StatusCode;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(fmt = "Error: {}", _1)]
pub struct Error(pub StatusCode, pub Cow<'static, str>);
