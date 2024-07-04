use std::future::Future;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{layout::Table, menu::MenuItem};

#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
    #[error(
        "A required '{kind}' ('{value}') is not present and cannot be mapped to the repository."
    )]
    ReferenceNotFound { kind: &'static str, value: String },
}

pub type Result<T> = std::result::Result<T, OrderingError>;
#[derive(Debug)]
pub struct Order<'a> {
    pub id: Option<u32>,
    pub table: &'a Table,
    pub menu_item: &'a MenuItem,
    pub time_placed: DateTime<Utc>,
}

pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;
pub trait Repository<'a> {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<Order>>> + Send;
    fn get(&self, id: u32) -> impl Future<Output = RepoResult<Order>> + Send;

    fn create(
        &mut self,
        item: Order<'a>,
        quantity: u32,
    ) -> impl Future<Output = RepoResult<()>> + Send;
    fn remove(&mut self, item: Order<'a>) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: Order<'a>) -> impl Future<Output = RepoResult<()>> + Send;
}

pub async fn place_order<'a, T: Repository<'a>>(
    repo: &mut T,
    table: &'a Table,
    item: &'a MenuItem,
    quantity: u32,
) -> Result<()> {
    if table.id.is_none() {
        Err(OrderingError::ReferenceNotFound {
            kind: "Table",
            value: "Unknown".to_string(),
        })
    } else if item.id.is_none() {
        Err(OrderingError::ReferenceNotFound {
            kind: "Item",
            value: item.name.clone(),
        })
    } else {
        repo.create(
            Order {
                id: None,
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
}
