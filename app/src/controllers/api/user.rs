use actix_http::StatusCode;

use actix_web::{
    web::{self, Json, ServiceConfig},
    HttpRequest, HttpResponse,
};

use shared::models::user::{CreateUser, SignInUser, UpdateUser, User};

use crate::{
    controllers::common::{self, AuthUser},
    repository::user::UserRepository,
    util::{error::Error, error_or::ErrorOr},
};

#[derive(Debug, derive_more::Display, derive_more::Error)]

pub enum UserError {
    #[display(fmt = "Invalid email or password provided. Try again.")]
    InvalidEmailOrPassword,
}

impl From<UserError> for Error {
    fn from(error: UserError) -> Self {
        match error {
            UserError::InvalidEmailOrPassword => Error::External(
                StatusCode::UNAUTHORIZED,
                error.to_string().into(),
            ),
        }
    }
}

pub fn service<R: UserRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/v1/users")
            .route("/login", web::post().to(login::<R>))
            .route("/register", web::post().to(register::<R>))
            .route("", web::get().to(get::<R>))
            .route("", web::put().to(put::<R>))
            .route("", web::delete().to(delete::<R>)),
    );
}

async fn login<R: UserRepository>(
    request: HttpRequest,
    login_user: web::Json<SignInUser>,
    repo: web::Data<R>,
) -> ErrorOr<HttpResponse> {
    let user = repo
        .get_user_by_email(&login_user.email)
        .await
        .0
        .map_err(|_| UserError::InvalidEmailOrPassword)?;

    common::login(&request, &user, &login_user).await?;

    HttpResponse::Ok().finish().into()
}

async fn register<R: UserRepository>(
    mut create_user: web::Json<CreateUser>,
    repo: web::Data<R>,
) -> ErrorOr<HttpResponse> {
    create_user.password = common::hash_password(&create_user.password).await?;

    repo.create_user(&create_user).await?;

    HttpResponse::Ok().finish().into()
}

async fn get<R: UserRepository>(
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<Json<User>> {
    repo.get_session_user(&user.id).await.0.map(Json).into()
}

async fn put<R: UserRepository>(
    update_user: web::Json<UpdateUser>,
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<HttpResponse> {
    repo.update_user(&update_user, &user.id).await?;

    HttpResponse::Ok().finish().into()
}

async fn delete<R: UserRepository>(
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<HttpResponse> {
    repo.delete_user(&user.id).await?;

    HttpResponse::Ok().finish().into()
}
