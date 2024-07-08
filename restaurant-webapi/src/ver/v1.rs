use axum::{routing::get, Router};

use super::VersionedApi;

pub fn create() -> VersionedApi {
    VersionedApi::new("v1", Router::new().route("/foo", get(get_foo)))
}

async fn get_foo() -> &'static str {
    "foo v1!"
}
