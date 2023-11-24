use crate::errors::ApiError;
use crate::filters;
use askama::Template;
use axum::{
  extract::{Path, State},
  response::IntoResponse,
  routing::{get, post, put},
  Form, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Todo {
  id: String,
  text: String,
  done: bool,
}

type Store = Mutex<Vec<Todo>>;
type MainState = State<Arc<Store>>;

pub(crate) fn todos_service() -> Router {
  let initial_todos: Vec<Todo> = vec![
    Todo {
      id: Uuid::new_v4().to_string(),
      text: "Learn React".to_string(),
      done: false,
    },
    Todo {
      id: Uuid::new_v4().to_string(),
      text: "Learn Vim".to_string(),
      done: true,
    },
  ];

  let store = Arc::new(Mutex::new(initial_todos));

  Router::new()
    .route("/", get(get_todos).post(create_todo))
    .route("/:id", post(delete_todo).put(edit_todo))
    .route("/toggle/:id", put(toggle_todo))
    .route("/clear", post(clear_completed))
    .with_state(store)
}

#[derive(Template)]
#[template(path = "todos.html")]
struct TodoList {
  todos: Vec<Todo>,
  remaining_todos: usize,
}

async fn get_todos(State(store): MainState) -> impl IntoResponse {
  tracing::info!("fetching todos from in-memory store");

  let todos = store.lock().await.clone();
  let remaining_todos = todos.iter().filter(|todo| !todo.done).count();
  TodoList {
    todos,
    remaining_todos,
  }
}

#[derive(Template)]
#[template(path = "todo.html")]
struct TodoItem {
  todo: Todo,
  remaining_todos: usize,
}

async fn toggle_todo(Path(id): Path<String>, State(store): MainState) -> impl IntoResponse {
  let mut todos = store.lock().await;

  tracing::info!("trying to toggle todo: {id}");

  todos
    .iter_mut()
    .find(|todo| todo.id == id)
    .map(|todo| {
      todo.done = !todo.done;
      todo.clone()
    })
    .map(|todo| {
      let remaining_todos = todos.iter().filter(|todo| !todo.done).count();

      TodoItem {
        todo,
        remaining_todos,
      }
      .into_response()
    })
    .unwrap_or(ApiError::TodoNotFound(id).into_response())
}

#[derive(Template)]
#[template(path = "todo-count.html")]
struct RemainingTodos {
  remaining_todos: usize,
}

async fn delete_todo(Path(id): Path<String>, State(store): MainState) -> impl IntoResponse {
  let mut todos = store.lock().await;
  let len = todos.len();

  tracing::info!("trying to delete todo: {id}");

  todos.retain(|todo| todo.id != id);

  if todos.len() != len {
    let remaining_todos = todos.iter().filter(|todo| !todo.done).count();
    RemainingTodos { remaining_todos }.into_response()
  } else {
    ApiError::TodoNotFound(id).into_response()
  }
}

#[derive(Deserialize, Serialize)]
struct CreateTodo {
  text: String,
}

async fn create_todo(State(store): MainState, Form(body): Form<CreateTodo>) -> impl IntoResponse {
  let mut todos = store.lock().await;
  tracing::info!("creating todo: {:?}", body.text);

  let new_todo = Todo {
    id: Uuid::new_v4().to_string(),
    text: body.text,
    done: false,
  };

  todos.push(new_todo.clone());

  let remaining_todos = todos.iter().filter(|todo| !todo.done).count();
  TodoItem {
    todo: new_todo,
    remaining_todos,
  }
}

async fn edit_todo(
  State(store): MainState,
  Path(id): Path<String>,
  Form(body): Form<CreateTodo>,
) -> impl IntoResponse {
  let mut todos = store.lock().await;
  let remaining_todos = todos.iter().filter(|todo| !todo.done).count();

  tracing::info!("trying to edit todo: {id}");

  todos
    .iter_mut()
    .find(|todo| todo.id == id)
    .map(|todo| {
      todo.text = body.text;
      TodoItem {
        todo: todo.clone(),
        remaining_todos,
      }
      .into_response()
    })
    .unwrap_or(ApiError::TodoNotFound(id).into_response())
}

async fn clear_completed(State(store): MainState) -> impl IntoResponse {
  let mut todos = store.lock().await;

  tracing::info!("clearing completed todos");

  todos.retain(|todo| !todo.done);
  let remaining_todos = todos.iter().filter(|todo| !todo.done).count();

  TodoList {
    todos: todos.clone(),
    remaining_todos,
  }
}

#[cfg(test)]
mod tests {
  use axum::http::StatusCode;
  use axum_test_helper::TestClient;

  async fn setup_tests() -> TestClient {
    let app = crate::router();
    TestClient::new(app)
  }

  #[tokio::test]
  async fn test_get_todos() {
    let client = setup_tests().await;

    let res = client.get("/todos").send().await;

    assert_eq!(res.status(), StatusCode::OK);

    let todos: String = res.text().await;
    assert_eq!(todos.contains("Learn React"), true);
    assert_eq!(todos.contains("Learn Vim"), true);
  }
}
