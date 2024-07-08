use axum::Router;

use super::VersionedApi;

mod menu_items;
mod orders;
mod tables;

pub fn create() -> VersionedApi {
    let router = Router::new()
        .nest("/orders", orders::create())
        .nest("/menu_items", menu_items::create())
        .nest("/tables", tables::create());

    VersionedApi::new("v1", router)
}
