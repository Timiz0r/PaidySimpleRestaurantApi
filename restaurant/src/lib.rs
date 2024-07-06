use std::ops::{Deref, DerefMut};

pub mod layout;
pub mod mem_repo;
pub mod menu;
pub mod order;

#[derive(Clone)]
pub struct RepoItem<T>(pub u32, pub T);

impl<T> RepoItem<T> {
    pub fn id(&self) -> u32 {
        self.0
    }
    pub fn item(&self) -> &T {
        &self.1
    }
    pub fn item_mut(&mut self) -> &mut T {
        &mut self.1
    }
}

impl<T> Deref for RepoItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> DerefMut for RepoItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RepoItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}
