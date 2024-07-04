use std::future::Future;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{layout::Table, menu::MenuItem, RepoItem};

#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, OrderingError>;
#[derive(Debug)]
pub struct Order<'a> {
    pub table: &'a RepoItem<Table>,
    pub menu_item: &'a RepoItem<MenuItem>,
    pub time_placed: DateTime<Utc>,
}

pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;
pub trait Repository<'a> {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoItem<Order>>>> + Send;
    fn get(&self, id: u32) -> impl Future<Output = RepoResult<RepoItem<Order>>> + Send;

    fn create(
        &mut self,
        item: Order<'a>,
        quantity: u32,
    ) -> impl Future<Output = RepoResult<()>> + Send;
    fn remove(&mut self, item: RepoItem<Order<'a>>) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: RepoItem<Order<'a>>) -> impl Future<Output = RepoResult<()>> + Send;
}

pub async fn place_order<'a, T: Repository<'a>>(
    repo: &mut T,
    table: &'a RepoItem<Table>,
    item: &'a RepoItem<MenuItem>,
    quantity: u32,
) -> Result<()> {
    repo.create(
        Order {
            table,
            menu_item: item,
            time_placed: Utc::now(),
        },
        quantity,
    )
    .await
    .map_err(OrderingError::RepoOperation)
    .and(Ok(()))
}
