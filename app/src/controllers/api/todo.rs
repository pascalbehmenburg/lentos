use crate::{
    controllers::common::AuthUser, repository::todo::TodoRepository,
    util::error_or::ErrorOr,
};
use actix_web::{
    web::{self, Json, ServiceConfig},
    HttpResponse,
};
use shared::models::todo::{CreateTodo, Todo, UpdateTodo};

pub fn service<R: TodoRepository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/v1/todos")
            .route("/{todo_id}", web::get().to(get::<R>))
            .route("", web::get().to(get_all::<R>))
            .route("", web::put().to(put::<R>))
            .route("/{todo_id}", web::delete().to(delete::<R>))
            .route("", web::post().to(post::<R>)),
    );
}

async fn get_all<R: TodoRepository>(
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<Json<Vec<Todo>>> {
    let res = repo.get_todos(&user.id).await?;
    Json(res).into()
}

async fn get<R: TodoRepository>(
    todo_id: web::Path<i64>,
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<Json<Todo>> {
    let todo = repo.get_todo(&todo_id, &user.id).await?;
    Json(todo).into()
}

async fn post<R: TodoRepository>(
    repo: web::Data<R>,
    create_todo: web::Json<CreateTodo>,
    user: AuthUser,
) -> ErrorOr<HttpResponse> {
    repo.create_todo(&create_todo, &user.id).await?;
    HttpResponse::Ok().finish().into()
}

async fn put<R: TodoRepository>(
    repo: web::Data<R>,
    update_todo: web::Json<UpdateTodo>,
    user: AuthUser,
) -> ErrorOr<HttpResponse> {
    repo.update_todo(&update_todo, &user.id).await?;
    HttpResponse::Ok().finish().into()
}

async fn delete<R: TodoRepository>(
    todo_id: web::Path<i64>,
    repo: web::Data<R>,
    user: AuthUser,
) -> ErrorOr<HttpResponse> {
    repo.delete_todo(&todo_id, &user.id).await?;
    HttpResponse::Ok().finish().into()
}
