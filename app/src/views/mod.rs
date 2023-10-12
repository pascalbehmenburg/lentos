pub use askama::Template;
use derive_more::Constructor;

#[derive(Template)]
#[template(path = "error.html")]
pub struct Error<'a> {
    pub title: &'a str,
    pub code: i32,
    pub msg: &'a str,
}

impl<'a> Error<'a> {
    pub fn new(title: &'a str, code: i32, msg: &'a str) -> Self {
        Self { title, code, msg }
    }
}

#[derive(Template, Constructor)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub title: &'a str,
}

#[derive(Template, Constructor)]
#[template(path = "login.html")]
pub struct Login {}
