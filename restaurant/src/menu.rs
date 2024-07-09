use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;

//TODO: want to try getting rid of the anyhow dependency
//for this kind of application, it's fine, but, if possible, want to find a different way for fun

#[derive(Error, Debug)]
pub enum MenuError {
    #[error("An error occurred when interacting with the repository.")]
    RepoOperation(#[from] anyhow::Error),

    #[error("Item '{item_name}' lacks an id and so cannot be mapped to repository.")]
    NoId { item_name: String },
}
type Result<T> = std::result::Result<T, MenuError>;
pub type RepoResult<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Minutes(pub u32);

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    // considered having the name be the key
    // but that would make name changes awkward
    pub name: String,
    // previously used chrono::TimeDelta, but it doesnt support serialization by default
    // in practice, basically every individual item in a restaurant should cook in minutes, so this actually works well
    pub cook_time: Minutes,
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
pub type RepoItem = crate::RepoItem<Item, Id>;

pub trait Repository {
    fn get_all(&self) -> impl Future<Output = RepoResult<Vec<RepoItem>>> + Send;
    fn get(&self, id: Id) -> impl Future<Output = RepoResult<RepoItem>> + Send;

    fn create(&mut self, item: Item) -> impl Future<Output = RepoResult<RepoItem>> + Send;
    fn remove(&mut self, id: Id) -> impl Future<Output = RepoResult<()>> + Send;
    fn update(&mut self, item: RepoItem) -> impl Future<Output = RepoResult<()>> + Send;
}

pub async fn get_all<T: Repository>(repo: &T) -> Result<Vec<RepoItem>> {
    repo.get_all().await.map_err(MenuError::RepoOperation)
}

pub async fn get<T: Repository>(repo: &T, id: Id) -> Result<RepoItem> {
    repo.get(id).await.map_err(MenuError::RepoOperation)
}

impl RepoItem {
    // NOTE: would be a series of functions that encompass the types of operations we'd want for a menu
    // set_cook_time is implemented as an example, not that it would work without a fully implemented MenuRepository

    // we could hypothetically have a MenuRepository be an optional member of MenuItem,
    // but lifetimes would get more complicated. still, since it's probably rather viable for the MenuRepository
    // to have a static lifetime, it might work out okay.
    pub async fn set_cook_time<T: Repository>(mut self, repo: &mut T, d: Minutes) -> Result<()> {
        if self.cook_time == d {
            Ok(())
        } else {
            self.cook_time = d;

            // we map_err for future-proofing, in case we use anyhow in other errors
            repo.update(self).await.map_err(MenuError::RepoOperation)
        }
    }

    pub async fn get<T: Repository>(&self, repo: &T, id: Id) -> Result<RepoItem> {
        repo.get(id).await.map_err(MenuError::RepoOperation)
    }
}
