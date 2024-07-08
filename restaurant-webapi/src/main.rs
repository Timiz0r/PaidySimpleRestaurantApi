use std::sync::{Arc, RwLock};

use axum::{
    body::Body, extract::Request, http::StatusCode, response::Response, routing::any, Extension,
    Router,
};
use restaurant::{layout, mem_repo::InMemoryRepository, menu, order};
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

type Table<T> = Arc<RwLock<InMemoryRepository<T>>>;
#[derive(Clone)]
pub(crate) struct Database {
    menu: Table<menu::Item>,
    tables: Table<layout::Table>,
    orders: Table<order::Order>,
}

impl Default for Database {
    fn default() -> Self {
        // we do a bunch of unwraps to panic if necessary, but they should all succeed.

        let mut menu = InMemoryRepository::new();
        menu.create(menu::Item {
            name: "Pasta".to_string(),
            cook_time: menu::Minutes(12),
        })
        .unwrap();
        menu.create(menu::Item {
            name: "Sandwich".to_string(),
            cook_time: menu::Minutes(5),
        })
        .unwrap();
        menu.create(menu::Item {
            name: "味噌カツ丼".to_string(),
            cook_time: menu::Minutes(12),
        })
        .unwrap();
        menu.create(menu::Item {
            name: "和風パフェ".to_string(),
            cook_time: menu::Minutes(8),
        })
        .unwrap();

        let mut tables = InMemoryRepository::new();
        for _ in 0..15 {
            tables.create(layout::Table {}).unwrap();
        }

        Self {
            menu: Arc::new(RwLock::new(menu)),
            tables: Arc::new(RwLock::new(tables)),
            orders: Arc::new(RwLock::new(InMemoryRepository::new())),
        }
    }
}

impl menu::Repository for Database {
    async fn get_all(&self) -> menu::RepoResult<Vec<menu::RepoItem>> {
        Ok(self.menu.read().unwrap().items().clone())
    }

    async fn get(&self, _id: u32) -> menu::RepoResult<menu::RepoItem> {
        unimplemented!()
    }

    async fn create(&mut self, _item: menu::Item) -> menu::RepoResult<menu::RepoItem> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: menu::RepoItem) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: menu::RepoItem) -> menu::RepoResult<()> {
        unimplemented!()
    }
}

impl layout::TableRepository for Database {
    async fn get_all(&self) -> layout::RepoResult<Vec<layout::RepoTable>> {
        Ok(self.tables.read().unwrap().items().clone())
    }

    async fn create(&mut self, _item: layout::Table) -> layout::RepoResult<layout::RepoTable> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: layout::RepoTable) -> layout::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: layout::RepoTable) -> layout::RepoResult<()> {
        unimplemented!()
    }
}

impl order::Repository for Database {
    async fn get_all(&self) -> order::RepoResult<Vec<order::RepoOrder>> {
        self.orders
            .read()
            .unwrap()
            .get_all()
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_table(
        &self,
        table: layout::RepoTable,
    ) -> order::RepoResult<Vec<order::RepoOrder>> {
        let results = self
            .orders
            .read()
            .unwrap()
            .items()
            .iter()
            .filter(|o| o.table.id() == table.id())
            .cloned()
            .collect::<Vec<order::RepoOrder>>();
        if results.is_empty() {
            Err(anyhow::anyhow!(
                "No orders found for table '{}'.",
                table.id()
            ))
        } else {
            Ok(results)
        }
    }

    async fn get(&self, id: u32) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .read()
            .unwrap()
            .get(id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create(&mut self, item: order::Order) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .create(item)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn remove(&mut self, item: order::RepoOrder) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .remove(item)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn update(&mut self, item: order::RepoOrder) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .update(item)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
