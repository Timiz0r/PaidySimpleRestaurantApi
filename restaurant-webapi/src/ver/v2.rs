use axum::{http::StatusCode, routing::any, Router};

use super::VersionedApi;

pub fn create() -> VersionedApi {
    VersionedApi::new("v2", Router::new().route("/*path", any(any_root)))
}

async fn any_root() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Use v1")
}
