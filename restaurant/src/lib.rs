use std::ops::{Deref, DerefMut};

use serde::Serialize;

pub mod layout;
pub mod mem_repo;
pub mod menu;
pub mod order;

#[derive(Clone, Serialize)]
pub struct RepoItem<T> {
    id: u32,

    #[serde(flatten)]
    item: T,
}

impl<T> RepoItem<T> {
    pub fn new(id: u32, item: T) -> RepoItem<T> {
        RepoItem { id, item }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn item(&self) -> &T {
        &self.item
    }
    pub fn item_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T> Deref for RepoItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for RepoItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RepoItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}
