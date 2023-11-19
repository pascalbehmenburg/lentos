use actix_web::web;

use crate::repository::{
    todo::PostgresTodoRepository, user::PostgresUserRepository,
};

pub mod health;
pub mod todo;
pub mod user;

pub fn service(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(health::service)
            .configure(todo::service::<PostgresTodoRepository>)
            .configure(user::service::<PostgresUserRepository>),
    );
}
