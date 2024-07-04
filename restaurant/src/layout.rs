use std::future::Future;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, LayoutError>;
#[derive(Debug)]
pub struct Table {
    pub id: Option<u32>,
}

pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;
pub trait TableRepository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<Table>>> + Send;

    fn create(&mut self, item: Table) -> impl Future<Output = RepoResult<Table>> + Send;
    fn remove(&mut self, item: Table) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: Table) -> impl Future<Output = RepoResult<Table>> + Send;
}

// see menu module for design notes

pub async fn get_tables<T: TableRepository>(repo: &T) -> Result<Vec<Table>> {
    repo.get_all().await.map_err(LayoutError::RepoOperation)
}
