use crate::{layout, menu, RepoItem};
use chrono::{DateTime, Utc};
use futures::Future;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// TODO: concurrent collections

#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
    #[error("Unable to find order {0:?}")]
    OrderNotFound(Id),
}
pub type Result<T> = std::result::Result<T, OrderingError>;
pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub table: layout::RepoTable,
    pub menu_item: menu::RepoItem,
    pub time_placed: DateTime<Utc>,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(pub u32);
impl From<u32> for Id {
    fn from(value: u32) -> Self {
        Id(value)
    }
}
impl From<Id> for u32 {
    fn from(value: Id) -> Self {
        value.0
    }
}
pub type RepoOrder = RepoItem<Order, Id>;

pub trait Repository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoOrder>>> + Send;
    fn get(&self, id: Id) -> impl Future<Output = RepoResult<RepoOrder>> + Send;

    fn create(&mut self, item: Order) -> impl Future<Output = RepoResult<RepoOrder>> + Send;
    fn remove(&mut self, id: Id) -> impl Future<Output = RepoResult<RepoOrder>> + Send;
    fn update(&mut self, item: RepoOrder) -> impl Future<Output = RepoResult<RepoOrder>> + Send;

    fn get_table(
        &self,
        table_id: layout::TableId,
    ) -> impl Future<Output = RepoResult<Vec<RepoOrder>>> + Send;
    fn remove_table_orders(
        &self,
        table_id: layout::TableId,
    ) -> impl Future<Output = RepoResult<Vec<RepoOrder>>> + Send;
}

pub async fn get_table<T: Repository>(
    repo: &T,
    table_id: layout::TableId,
) -> Result<Vec<RepoOrder>> {
    repo.get_table(table_id)
        .await
        .map_err(OrderingError::RepoOperation)
}

pub async fn place<T: Repository>(
    repo: &mut T,
    table: layout::RepoTable,
    menu_item: menu::RepoItem,
    quantity: u32,
) -> Result<RepoOrder> {
    repo.create(Order {
        table,
        menu_item,
        time_placed: Utc::now(),
        quantity,
    })
    .await
    .map_err(OrderingError::RepoOperation)
}

pub async fn set_quantity<T: Repository>(repo: &mut T, id: Id, quantity: u32) -> Result<RepoOrder> {
    if quantity == 0 {
        return cancel(repo, id).await;
    }

    if let Ok(mut order) = repo.get(id).await {
        order.quantity = quantity;

        repo.update(order)
            .await
            .map_err(OrderingError::RepoOperation)
    } else {
        Err(OrderingError::OrderNotFound(id))
    }
}

pub async fn cancel<T: Repository>(repo: &mut T, id: Id) -> Result<RepoOrder> {
    repo.remove(id).await.map_err(OrderingError::RepoOperation)
}

pub async fn clear_table<T: Repository>(
    repo: &mut T,
    table_id: layout::TableId,
) -> Result<Vec<RepoOrder>> {
    repo.remove_table_orders(table_id)
        .await
        .map_err(OrderingError::RepoOperation)
}
