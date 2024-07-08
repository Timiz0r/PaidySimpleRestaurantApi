use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use restaurant::layout;

use crate::Database;

pub fn create() -> Router {
    Router::new().route("/", get(tables_get))
}

async fn tables_get(Extension(db): Extension<Database>) -> Result<impl IntoResponse, StatusCode> {
    layout::get_tables(&db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
