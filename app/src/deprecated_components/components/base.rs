use maud::{Markup, html, DOCTYPE};

use crate::{controllers::common::AuthUser, util::error_or::ErrorOr};

use super::auth::login_view;


pub fn header_view(title: &str) -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta http-equiv="X-UA-Compatible" content="ie=edge";
            meta name="theme-color" content="#000000";
            title { (title) " | Lentos" }
            link rel="stylesheet" type="text/css" href="/static/tailwind.css";
            link rel="icon" href="/static/favicon.svg";
            link rel="mask-icon" color="#000000" href="/static/favicon.svg";
            link rel="apple-touch-icon" href="/static/favicon.svg";
        }
    }
}

pub async fn base_view(title: &str, block: Markup) -> Markup {
    html! {
        html class="h-full scroll-smooth" lang="en" dir="ltr" {
            (DOCTYPE)
            (header_view(title))
            body class="font-sans antialiased" {
                main {
                    (block)
                }
            }
            script src="/static/htmx.min.js";
            script {
                "htmx.logAll();"
            }
        }
    }
}