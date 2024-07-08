use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;

use crate::RepoItem;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
}
pub type Result<T> = std::result::Result<T, LayoutError>;
pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug, Clone, Serialize)]
pub struct Table {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TableId(pub u32);
impl From<u32> for TableId {
    fn from(value: u32) -> Self {
        TableId(value)
    }
}
impl From<TableId> for u32 {
    fn from(value: TableId) -> Self {
        value.0
    }
}

pub type RepoTable = RepoItem<Table, TableId>;

pub trait TableRepository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoTable>>> + Send;
    fn get(&self, id: TableId) -> impl Future<Output = RepoResult<RepoTable>> + Send;

    fn create(&mut self, item: Table) -> impl Future<Output = RepoResult<RepoTable>> + Send;
    fn remove(&mut self, id: TableId) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: RepoTable) -> impl Future<Output = RepoResult<()>> + Send;
}

// see menu module for design notes. not fully implementing because not necessary for this project.

pub async fn get_tables<T: TableRepository>(repo: &T) -> Result<Vec<RepoTable>> {
    repo.get_all().await.map_err(LayoutError::RepoOperation)
}

pub async fn get<T: TableRepository>(repo: &T, id: TableId) -> Result<RepoTable> {
    repo.get(id).await.map_err(LayoutError::RepoOperation)
}
