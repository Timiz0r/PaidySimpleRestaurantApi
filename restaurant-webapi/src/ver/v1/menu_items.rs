use crate::Database;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use restaurant::menu;

pub fn create() -> Router {
    Router::new().route("/menu_items", get(get_all))
}

async fn get_all(Extension(db): Extension<Database>) -> Result<impl IntoResponse, StatusCode> {
    menu::get(&db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
