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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match (
        layout::get(&db, order.table_id).await,
        menu::get(&db, order.item_id).await,
    ) {
        (Ok(_), Ok(_)) => order::place(&mut db, order.table_id, order.item_id, order.quantity)
            .await
            .map(Json)
            .map_err(|e| {
                // TODO: to make things more readable, shoving these in a tuple struct should hopefully work
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create order: {:?}", e),
                )
            }),
        (Ok(_), Err(_)) => Err((
            (StatusCode::BAD_REQUEST),
            format!("Table '{:?}' not found.", order.table_id),
        )),
        (Err(_), Ok(_)) => Err((
            (StatusCode::BAD_REQUEST),
            format!("Menu item '{:?}' not found.", order.item_id),
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
    // rather than many repo calls, we'll just get them all,
    // since a restaurant probably doesn't have many menu items
    // could also turn it into a map if necessary
    let menu = match menu::get_all(&db).await {
        Ok(menu) => menu,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get menu: {:?}", e),
            ));
        }
    };

    let table = match layout::get(&db, table_id).await {
        Ok(table) => table,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to find table '{:?}': {:?}", table_id, e),
            ));
        }
    };

    let orders = match order::get_table(&db, table_id).await {
        Ok(orders) => orders,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get orders for table '{:?}': {:?}", table_id, e),
            ));
        }
    };

    // what should we do in this case? it's actually a case for denormalization of stored orders,
    // since there's probably no legitimate scenario where we want orders auto-removed when associated
    // menu items are removed. staff could always just remove the item in the off chance this happens.
    //
    // the problem is that, in the current design, order::Repository wants a menu::Id and has no other way to
    // turn it into full menu::Items.
    // TODO: could be doable. give it a try.
    let missing_items: Vec<String> = orders
        .iter()
        .filter(|o| !menu.iter().any(|i| i.id() == o.menu_item_id))
        .map(|o| format!("order '{:?}', menu item '{:?}", o.id(), o.menu_item_id))
        .collect();
    if !missing_items.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "The following orders have missing menu items: {}",
                missing_items.join("; ")
            ),
        ));
    }

    Ok(Json(
        orders
            .iter()
            .map(|o| {
                let item = menu
                    .iter()
                    .find(|i| i.id() == o.menu_item_id)
                    .expect("Verified to exist.");
                let remaining = TimeDelta::minutes(item.item().cook_time.0.into())
                    - (Utc::now() - o.time_placed);

                OrderDetails {
                    id: o.id(),
                    table: table.clone(),
                    item: item.clone(),
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
