mod errors;
mod filters;
mod todos;

use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};
use color_eyre::Result;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub(crate) fn router() -> Router {
  Router::new()
    .route("/", get(home))
    .nest_service("/static", ServeDir::new("public"))
    .nest("/todos", todos::todos_service())
}

#[tokio::main]
async fn main() -> Result<()> {
  // initialise tracing
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    // .with_max_level(tracing::Level::DEBUG)
    .with_line_number(true)
    .init();

  let app = router();
  let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();
  let addr = listener.local_addr().unwrap().to_string();

  tracing::info!("App running on http://{}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}

async fn home() -> impl IntoResponse {
  HelloTemplate
}

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate;
