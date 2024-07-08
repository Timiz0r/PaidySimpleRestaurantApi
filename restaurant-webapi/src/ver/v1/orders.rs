use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use restaurant::{layout, menu, order};
use serde::Deserialize;

use crate::Database;

pub fn create() -> Router {
    Router::new()
        .route("/orders", post(orders_post))
        .route("/table/:tableid/orders", get(table_orders_get))
        .route("/orders/:id/setquantity", post(orders_setquantity))
        .route("/orders/:id", delete(orders_delete))
}

#[derive(Debug, Deserialize)]
struct CreateOrder {
    table_id: layout::TableId,
    item_id: menu::Id,
    quantity: u32,
}

async fn orders_post(
    Extension(mut db): Extension<Database>,
    Json(order): Json<CreateOrder>,
) -> Result<impl IntoResponse, StatusCode> {
    match (
        layout::TableRepository::get(&db, order.table_id).await,
        menu::Repository::get(&db, order.item_id).await,
    ) {
        (Ok(_), Ok(_)) => order::place(&mut db, order.table_id, order.item_id, order.quantity)
            .await
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn table_orders_get(
    Extension(db): Extension<Database>,
    Path((_, table_id)): Path<(String, layout::TableId)>,
) -> Result<impl IntoResponse, StatusCode> {
    match layout::TableRepository::get(&db, table_id).await {
        Ok(_) => order::get_table(&db, table_id)
            .await
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Deserialize)]
struct SetOrderQuantity {
    quantity: u32,
}

async fn orders_setquantity(
    Extension(mut db): Extension<Database>,
    Path((_, id)): Path<(String, order::Id)>,
    Json(SetOrderQuantity { quantity }): Json<SetOrderQuantity>,
) -> Result<impl IntoResponse, StatusCode> {
    order::set_quantity(&mut db, id, quantity)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn orders_delete(
    Extension(mut db): Extension<Database>,
    Path((_, id)): Path<(String, order::Id)>,
) -> Result<impl IntoResponse, StatusCode> {
    order::cancel(&mut db, id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
