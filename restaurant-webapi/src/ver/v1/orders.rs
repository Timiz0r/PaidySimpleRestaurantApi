use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, TimeDelta, Utc};
use restaurant::{layout, menu, order};
use serde::{Deserialize, Serialize};

use crate::Database;

pub fn create() -> Router {
    Router::new()
        .route("/orders", post(orders_post))
        .route("/orders/:id/setquantity", post(orders_setquantity))
        .route("/orders/:id", delete(orders_delete))
        .route("/table/:tableid/orders", get(table_orders_get))
        .route("/table/:tableid/clear", post(table_orders_clear))
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match (
        layout::get(&db, order.table_id).await,
        menu::get(&db, order.item_id).await,
    ) {
        (Ok(table), Ok(item)) => {
            order::place(&mut db, table, item, order.quantity)
                .await
                .map(Json)
                .map_err(|e| {
                    // TODO: to make things more readable, shoving these in a tuple struct should hopefully work
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create order: {:?}", e),
                    )
                })
        }
        (Ok(_), Err(_)) => Err((
            (StatusCode::BAD_REQUEST),
            format!("Menu item '{:?}' not found.", order.item_id),
        )),
        (Err(_), Ok(_)) => Err((
            (StatusCode::BAD_REQUEST),
            format!("Table '{:?}' not found.", order.table_id),
        )),
        (Err(_), Err(_)) => Err((
            (StatusCode::BAD_REQUEST),
            format!(
                "Table '{:?}' and menu item '{:?}' not found.",
                order.table_id, order.item_id
            ),
        )),
    }
}

#[derive(Debug, Serialize)]
struct OrderDetails {
    id: order::Id,
    table: layout::RepoTable,
    item: menu::RepoItem,
    time_placed: DateTime<Utc>,
    quantity: u32,
    estimated_minutes_remaining: menu::Minutes,
}

async fn table_orders_get(
    Extension(db): Extension<Database>,
    Path((_, table_id)): Path<(String, layout::TableId)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let orders = match order::get_table(&db, table_id).await {
        Ok(orders) => orders,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get orders for table '{:?}': {:?}", table_id, e),
            ));
        }
    };

    Ok(Json(
        orders
            .iter()
            .map(|o| {
                let remaining = TimeDelta::minutes((o.menu_item.cook_time.0 * o.quantity).into())
                    - (Utc::now() - o.time_placed);

                OrderDetails {
                    id: o.id(),
                    table: o.table.clone(),
                    item: o.menu_item.clone(),
                    time_placed: o.time_placed,
                    quantity: o.quantity,
                    estimated_minutes_remaining: menu::Minutes(
                        remaining.num_minutes().try_into().unwrap_or(0),
                    ),
                }
            })
            .collect::<Vec<OrderDetails>>(),
    ))
}

#[derive(Debug, Deserialize)]
struct SetOrderQuantity {
    quantity: u32,
}

async fn orders_setquantity(
    Extension(mut db): Extension<Database>,
    Path((_, id)): Path<(String, order::Id)>,
    Json(SetOrderQuantity { quantity }): Json<SetOrderQuantity>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    order::set_quantity(&mut db, id, quantity)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set quantity for order '{:?}': {:?}", id, e),
            )
        })
}

async fn orders_delete(
    Extension(mut db): Extension<Database>,
    Path((_, id)): Path<(String, order::Id)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    order::cancel(&mut db, id).await.map(Json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to cancel order '{:?}': {:?}", id, e),
        )
    })
}

async fn table_orders_clear(
    Extension(mut db): Extension<Database>,
    Path((_, table_id)): Path<(String, layout::TableId)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    order::clear_table(&mut db, table_id)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to clear table'{:?}': {:?}", table_id, e),
            )
        })
}
