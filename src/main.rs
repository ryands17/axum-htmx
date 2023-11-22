mod errors;
mod filters;
mod todos;

use std::net::SocketAddr;

use anyhow::Result;
use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(source = "{{ condition|yes_no(\"yes\", \"no\") }}", ext = "txt")]
struct YesNoFilterTemplate<'a> {
  condition: &'a bool,
}

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
  let addr = SocketAddr::from(([0, 0, 0, 0], 3001));

  tracing::info!("App running on http://{}", addr);

  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;

  Ok(())
}

async fn home() -> impl IntoResponse {
  HelloTemplate
}

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate;
