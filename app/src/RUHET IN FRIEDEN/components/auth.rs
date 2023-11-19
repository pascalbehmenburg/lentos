use actix_web::{web, HttpRequest};
use maud::{html, Markup};
use shared::models::user::{LoginUser, CreateUser};

use crate::{repository::{user::UserRepository}, util::error_or::ErrorOr, controllers::{common, api::user::UserError}};

use super::home::main_view;

pub fn service<R: UserRepository>(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(web::scope("/auth")
                    .route("/login", web::post().to(login::<R>))
                    .route("/login", web::get().to(login_view))
                    .route("/register", web::post().to(register::<R>))
                    .route("/register", web::get().to(register_view))
    );
}

async fn register<R: UserRepository>(
    _: HttpRequest,
    mut create_user: web::Form<CreateUser>,
    repo: web::Data<R>,
) -> ErrorOr<Markup> {
    create_user.password = common::hash_password(&create_user.password).await?;

    repo.create_user(&create_user).await?;

    // if done registering you may want to log in
    login_view().await.into()
}

pub async fn register_view() -> Markup {
    html! {
        form id="register-form" hx-swap="outerHTML" hx-post="/component/auth/register" {
            input   class="mb-4 shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 leading-tight focus:outline-none focus:shadow-outline" id="name" type="text" name="name" placeholder="Username" required;
            input   class="mb-4 shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 leading-tight focus:outline-none focus:shadow-outline" id="email" type="text" name="email" placeholder="Email" required;
            input   class="mb-6 shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 leading-tight focus:outline-none focus:shadow-outline" id="password" type="password" name="password" placeholder="Password" required;
            div class="flex flex-row justify-between items-center" {
                button  class=r#"bg-blue-500
                                dark:bg-blue-600
                                hover:bg-blue-700
                                dark:hover:bg-blue-800
                                text-white font-bold py-2 px-4
                                rounded focus:outline-none focus:shadow-outline"#
                        type="submit"
                {
                    "Register"
                }
                a hx-swap="outerHTML" hx-target="#register-form" hx-get="/component/auth/login" href="" {
                    "Already have an account?"
                }
            }
       }
    }
}

pub async fn login_view() -> Markup {
    html! {
        form id="login-form" hx-swap="outerHTML" hx-post="/component/auth/login" {
            input class="mb-4 shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 leading-tight focus:outline-none focus:shadow-outline" id="email" type="text" name="email" placeholder="Email" required;
            input class="mb-6 shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 leading-tight focus:outline-none focus:shadow-outline" id="password" type="password" name="password" placeholder="Password" required;
            div class="flex flex-row justify-between items-center" {
                button  class=r#"bg-blue-500
                                dark:bg-blue-600
                                hover:bg-blue-700
                                dark:hover:bg-blue-800
                                text-white font-bold py-2 px-4
                                rounded focus:outline-none focus:shadow-outline"#
                        type="submit"
                {
                    "Sign In"
                }
                a hx-swap="outerHTML" hx-target="#login-form" hx-get="/component/auth/register" href="" {
                    "Don't have an account?"
                }
            }
       }
    }
}



async fn login<R: UserRepository>(
    request: HttpRequest,
    login_user: web::Form<LoginUser>,
    repo: web::Data<R>,
) -> ErrorOr<Markup> {
    let user = repo
        .get_user_by_email(&login_user.email)
        .await
        .0
        .map_err(|_| UserError::InvalidEmailOrPassword)?;

    common::login(&request, &user, &login_user).await?;

    // if the above succeeds, we can display user information without extracting from cookie
    // though all requests that happen after this one have to use AuthUser
    // TODO consider adding a guard if possible
    main_view(&user).await.into()
}