use crate::Database;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use restaurant::menu;

pub fn create() -> Router {
    Router::new().route("/", get(get_all))
}

async fn get_all(Extension(repo): Extension<Database>) -> Result<impl IntoResponse, StatusCode> {
    menu::get(&repo)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
