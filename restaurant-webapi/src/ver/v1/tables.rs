use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use restaurant::layout;

use crate::Database;

pub fn create() -> Router {
    Router::new().route("/", get(get_all))
}

async fn get_all(Extension(repo): Extension<Database>) -> Result<impl IntoResponse, StatusCode> {
    layout::get_tables(&repo)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
