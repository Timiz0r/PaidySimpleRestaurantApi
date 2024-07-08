use axum::{
    body::Body, extract::Request, http::StatusCode, response::Response, routing::any, Extension,
    Router,
};
use restaurant::memdb::Database;
use tower::{Service, ServiceBuilder};

mod ver;

#[tokio::main]
async fn main() {
    // purposely putting this in main so that it can be moved to the below closure
    let mut versioned_apis = ver::create_services();
    let app = Router::new().route(
        "/api/*path",
        any(|request: Request| async move {
            if let Some(router) = request
                .headers()
                .get("x-api-version")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| versioned_apis.get_mut(v))
            {
                router.call(request).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Set 'x-api-version' header."))
                    .unwrap())
            }
        })
        .layer(ServiceBuilder::new().layer(Extension(Database::default()))),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(
        std::env::args()
            .nth(1)
            .unwrap_or("127.0.0.1:13981".to_string()),
    )
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap()
}
