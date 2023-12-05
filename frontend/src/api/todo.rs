use crate::handler::api_handler::{ApiHandler, BASE_URL};
use reqwest::StatusCode;
use shared::models::todo::{CreateTodo, Todo, UpdateTodo};

pub(crate) async fn create_todo(
    api_handler: &ApiHandler,
    create_todo: CreateTodo,
) {
    tracing::debug!("Trying to create a todo...");

    let response = api_handler.post("/todos", &create_todo).await;

    if !response.status().is_success() {
        tracing::error!(
            "Failed to create a todo. Server responded: {:?}",
            response
        );
    } else {
        tracing::debug!("Todo created.");
    }
}

pub(crate) async fn get_all_todos(api_handler: &ApiHandler) -> Vec<Todo> {
    tracing::debug!("Trying to get all todos...");

    let response = api_handler.get("/todos").await;

    if !response.status().is_success() {
        tracing::error!(
            "Failed to get all todos. Server responded: {:?}",
            response
        );
    } else {
        tracing::debug!("Got todos.");
    }

    let todos =
        response.json::<Vec<Todo>>().await.expect("Failed to parse response");

    tracing::debug!("Parsed todos: {:?}", todos);

    todos
}

pub(crate) async fn get_todo(api_handler: &ApiHandler, todo_id: &i64) -> Todo {
    tracing::debug!("Trying to get todo with id: {todo_id}...");

    let response = api_handler.get(&format!("/todos/{todo_id}")).await;

    if !response.status().is_success() {
        tracing::error!(
            "Failed to get todo with id: {todo_id}. Server responded: {:?}",
            response
        );
    } else {
        tracing::debug!("Got todo with id: {todo_id}.");
    }

    let todo = response.json::<Todo>().await.expect("Failed to parse response");

    tracing::debug!("Parsed todo: {:?}", todo);

    todo
}

pub(crate) async fn delete_todo(api_handler: &ApiHandler, todo_id: &i64) {
    tracing::debug!("Trying to delete todo with id: {todo_id}...");
    let response = api_handler
        .client
        .delete(&format!("{BASE_URL}/todos/{todo_id}"))
        .send()
        .await
        .expect("Failed to send request");

    if !response.status().is_success() {
        tracing::error!(
            "Failed to delete todo with id: {todo_id}. Server responded: {:?}",
            response
        );
    } else {
        tracing::debug!("Deleted todo with id: {todo_id}.");
    }
}

pub(crate) async fn update_todo(
    api_handler: &ApiHandler,
    update_todo: UpdateTodo,
) -> StatusCode {
    let todo_id = update_todo.id;
    tracing::debug!("Trying to update {update_todo:?}...");

    let response = api_handler
        .client
        .put(&format!("{BASE_URL}/todos"))
        .json(&update_todo)
        .send()
        .await
        .expect("Failed to send request");

    if !response.status().is_success() {
        tracing::error!(
            "Failed to update todo {todo_id}. Server responded: {:?}",
            response
        );
    } else {
        tracing::debug!(
            "Updated todo {todo_id}. Server responded: {:?}",
            response
        );
    }

    response.status()
}

// this does not ensure the correctness of the functions tested
// but provides a way to run them without the dioxus context
// also if they panic while testing something is wrong :)
mod tests {
    use super::*;
    use shared::models::todo::UpdateTodo;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn get_todo_test() {
        let api_handler = ApiHandler::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(get_todo(&api_handler, &1));
    }

    #[test]
    #[traced_test]
    fn get_all_todos_test() {
        let api_handler = ApiHandler::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(get_all_todos(&api_handler));
    }

    #[test]
    #[traced_test]
    fn create_todo_test() {
        let api_handler = ApiHandler::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let todo = CreateTodo {
            title: "Title".to_string(),
            description: "Description".to_string(),
        };
        rt.block_on(create_todo(&api_handler, todo));
    }

    #[test]
    #[traced_test]
    fn delete_todo_test() {
        let api_handler = ApiHandler::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let todo = CreateTodo {
            title: "Title".to_string(),
            description: "Description".to_string(),
        };
        // create a todo to delete
        rt.block_on(create_todo(&api_handler, todo));

        // get all todos to find the latest created todo afterwards and delete
        // it
        let todos = rt.block_on(get_all_todos(&api_handler));
        let todo_to_delete = todos.last().unwrap();
        rt.block_on(delete_todo(&api_handler, &todo_to_delete.id));
    }

    #[test]
    #[traced_test]
    fn update_todo_test() {
        let api_handler = ApiHandler::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let create_todo_data = CreateTodo {
            title: "Title".to_string(),
            description: "Description".to_string(),
        };

        // create a todo to update
        rt.block_on(create_todo(&api_handler, create_todo_data));

        let todos = rt.block_on(get_all_todos(&api_handler));

        let todo_to_update = todos.last().unwrap();

        let update_todo_data = UpdateTodo {
            id: todo_to_update.id,
            title: Some("Updated title".to_string()),
            description: Some("Updated description".to_string()),
            is_done: Some(true),
        };

        rt.block_on(update_todo(&api_handler, &update_todo_data.id));
    }
}
