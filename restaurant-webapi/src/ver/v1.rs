use axum::Router;

use super::VersionedApi;

mod menu_items;
mod orders;
mod tables;

pub fn create() -> VersionedApi {
    let router = Router::new()
        .merge(orders::create())
        .merge(menu_items::create())
        .merge(tables::create());

    VersionedApi::new("v1", router)
}
