use crate::{layout, menu, RepoItem};
use chrono::{DateTime, Utc};
use futures::Future;
use thiserror::Error;

// TODO: concurrent collections

#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, OrderingError>;
#[derive(Debug, Clone)]
pub struct Order {
    pub table: layout::RepoTable,
    pub menu_item: menu::RepoItem,
    pub time_placed: DateTime<Utc>,
    pub quantity: u32,
}
pub type RepoOrder = RepoItem<Order>;

pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;
pub trait Repository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoOrder>>> + Send;
    fn get_table(
        &self,
        table: layout::RepoTable,
    ) -> impl Future<Output = RepoResult<Vec<RepoOrder>>> + Send;
    fn get(&self, id: u32) -> impl Future<Output = RepoResult<RepoOrder>> + Send;

    fn create(&mut self, item: Order) -> impl Future<Output = RepoResult<RepoOrder>> + Send;
    fn remove(&mut self, item: RepoOrder) -> impl Future<Output = RepoResult<RepoOrder>> + Send;
    fn update(&mut self, item: RepoOrder) -> impl Future<Output = RepoResult<RepoOrder>> + Send;
}

pub async fn get<T: Repository>(repo: &T, table: layout::RepoTable) -> Result<Vec<RepoOrder>> {
    repo.get_table(table)
        .await
        .map_err(OrderingError::RepoOperation)
}

pub async fn place<T: Repository>(
    repo: &mut T,
    table: layout::RepoTable,
    item: menu::RepoItem,
    quantity: u32,
) -> Result<RepoOrder> {
    repo.create(Order {
        table,
        menu_item: item,
        time_placed: Utc::now(),
        quantity,
    })
    .await
    .map_err(OrderingError::RepoOperation)
}

pub async fn set_quantity<T: Repository>(
    repo: &mut T,
    order: RepoOrder,
    quantity: u32,
) -> Result<RepoOrder> {
    if quantity == 0 {
        return cancel(repo, order).await;
    }

    repo.update(order)
        .await
        .map_err(OrderingError::RepoOperation)
}

pub async fn cancel<T: Repository>(repo: &mut T, order: RepoOrder) -> Result<RepoOrder> {
    repo.remove(order)
        .await
        .map_err(OrderingError::RepoOperation)
}
