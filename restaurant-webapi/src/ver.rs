use std::collections::HashMap;

use axum::Router;

pub mod v1;
pub mod v2;

pub fn create_services() -> HashMap<&'static str, Router> {
    let mut result = HashMap::new();
    let mut add = |VersionedApi(v, r): VersionedApi| {
        result.insert(v, r);
    };
    add(v1::create());
    add(v2::create());

    result
}

#[derive(Clone)]
pub struct VersionedApi(&'static str, Router);

impl VersionedApi {
    pub fn new(ver: &'static str, router: Router) -> VersionedApi {
        VersionedApi(ver, Router::new().nest("/api", router))
    }
}
