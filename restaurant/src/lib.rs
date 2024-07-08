use std::ops::{Deref, DerefMut};

use serde::Serialize;

pub mod layout;
pub mod memdb;
pub mod menu;
pub mod order;

#[derive(Clone, Serialize)]
pub struct RepoItem<T, I: Copy + Clone + Serialize> {
    id: I,

    #[serde(flatten)]
    item: T,
}

impl<T, I: Copy + Clone + Serialize> RepoItem<T, I> {
    pub fn new(id: I, item: T) -> RepoItem<T, I> {
        RepoItem { id, item }
    }
    pub fn id(&self) -> I {
        self.id
    }
    pub fn item(&self) -> &T {
        &self.item
    }
    pub fn item_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T, I: Copy + Clone + Serialize> Deref for RepoItem<T, I> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T, I: Copy + Clone + Serialize> DerefMut for RepoItem<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T: std::fmt::Debug, I: Copy + Clone + Serialize> std::fmt::Debug for RepoItem<T, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}
