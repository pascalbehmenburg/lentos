use actix_web::{web, HttpRequest};
use color_eyre::eyre;
use maud::{html, Markup};
use shared::models::todo::{Todo, UpdateTodo, FormUpdateTodo};

use crate::{repository::{user::UserRepository, todo::TodoRepository}, controllers::common::AuthUser, util::{error_or::ErrorOr, error::Error}};



pub fn service<R: TodoRepository>(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(web::scope("/todo")
                    .route("/list", web::get().to(list::<R>))
                    .route("/todo-edit", web::get().to(todo_edit))
                    .route("/description-edit", web::get().to(description_edit))
                    .route("/update", web::put().to(update::<R>))
    );
}

pub async fn todo_edit(description: web::Form<String>) -> Markup {
    html! {
        input type="text"
            name="title"
            value=(title)
            class="text-lg font-semibold"
            placeholder="Title"
            required
            autofocus
            autocomplete="off";

        input type="text"
            name="description"
            value=(description)
            class="text-lg font-semibold"
            placeholder="Description"
            required
            autofocus
            autocomplete="off"
            hx-put="/component/todo/update"
            hx-trigger="onEnter";
    }
}

pub async fn title(title: &str) -> Markup {
    html! {
        p   class="text-lg font-semibold"
            hx-trigger="click"
            hx-get="/component/todo/todo-edit"
            hx-swap="nearest div" {
            (title)
        }
    }
}

pub async fn description(description: &str) -> Markup {
    html! {
        p   class="text-sm text-gray-500"
            hx-trigger="click"
            hx-get="/component/todo/todo-edit"
            hx-swap="nearest div" {
            (description)
        }
    }
}

pub async fn todo_component(todo: &Todo) -> Markup {
    html! {
        div id=(todo.id) class="flex max-w-xs items-center space-x-2 rounded-lg bg-gray-100 p-4 shadow-lg todo" {
            form method="PUT" {
                input type="checkbox"
                    name="is_done"
                    class="flex h-6 w-6 cursor-pointer items-center justify-center rounded border border-gray-300"
                    hx-trigger="click"
                    hx-put="/component/todo/update"
                    hx-swap="none";
                div class="flex flex-col" {
                    div {
                        (title(&todo.title).await)
                        (description(&todo.description).await)
                    }
                }
            }
        }
    }
}

pub async fn todo_list(todos: Vec<Todo>) -> Markup {
    html! {
        p { "Todo list" }
        ul {
            @for todo in &todos {
                li { (todo_component(todo).await) }
            }
        }
    }
}

pub async fn list<R: TodoRepository>(
    repo: web::Data<R>,
    user: AuthUser
) -> ErrorOr<Markup> {

    let todos = repo.get_todos(&user.id).await?;
    todo_list(todos).await.into()
}

pub async fn update<R: TodoRepository>(
    update_todo: web::Form<FormUpdateTodo>,
    repo: web::Data<R>,
    user: AuthUser
) -> ErrorOr<Markup> {
    let update_todo: UpdateTodo = update_todo.0.into();
    let updated_todo = repo.update_todo(&update_todo, &user.id).await?;
    todo_component(&updated_todo).await.into()
}

