use serde::Serialize;
use std::future::Future;
use thiserror::Error;

use crate::RepoItem;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, LayoutError>;
#[derive(Debug, Clone, Serialize)]
pub struct Table {}
pub type RepoTable = RepoItem<Table>;

pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;
pub trait TableRepository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoTable>>> + Send;

    fn create(&mut self, item: Table) -> impl Future<Output = RepoResult<RepoTable>> + Send;
    fn remove(&mut self, item: RepoTable) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: RepoTable) -> impl Future<Output = RepoResult<()>> + Send;
}

// see menu module for design notes. not fully implementing because not necessary for this project.

pub async fn get_tables<T: TableRepository>(repo: &T) -> Result<Vec<RepoTable>> {
    repo.get_all().await.map_err(LayoutError::RepoOperation)
}
