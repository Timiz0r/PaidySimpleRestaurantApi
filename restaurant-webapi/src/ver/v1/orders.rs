use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;

pub fn create() -> Router {
    Router::new().route("/orders", post(orders_post))
    // .route("/orders", post(orders_post))
}

async fn orders_root() -> impl IntoResponse {
    "foo v1!"
}

#[derive(Debug, Deserialize)]
struct CreateOrder {
    table_id: u32,
    item_id: u32,
    quantity: u32,
}

async fn orders_post(Json(order): Json<CreateOrder>) -> Response {
    //order::place()
    Response::default()
}
