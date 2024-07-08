use axum::{
    body::Body, extract::Request, http::StatusCode, response::Response, routing::any, Extension,
    RequestPartsExt, Router,
};
use restaurant::{layout, memdb::Database, menu};
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
        .layer(ServiceBuilder::new().layer(Extension(create_database()))),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(
        std::env::args()
            .nth(1)
            .unwrap_or("127.0.0.1:13982".to_string()),
    )
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap()
}

fn create_database() -> Database {
    let tables = (1..15)
        .map(|id| layout::RepoTable::new(id.into(), layout::Table {}))
        .collect();
    let menu = vec![
        menu::RepoItem::new(
            1.into(),
            menu::Item {
                name: "Pasta".to_string(),
                cook_time: menu::Minutes(12),
            },
        ),
        menu::RepoItem::new(
            2.into(),
            menu::Item {
                name: "Sandwich".to_string(),
                cook_time: menu::Minutes(5),
            },
        ),
        menu::RepoItem::new(
            3.into(),
            menu::Item {
                name: "味噌カツ丼".to_string(),
                cook_time: menu::Minutes(15),
            },
        ),
        menu::RepoItem::new(
            4.into(),
            menu::Item {
                name: "和風パフェ".to_string(),
                cook_time: menu::Minutes(8),
            },
        ),
    ];
    Database::new(menu, tables, vec![])
}
