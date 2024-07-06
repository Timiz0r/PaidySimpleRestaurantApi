use std::{clone::Clone, collections::HashSet, result::Result, sync::atomic::AtomicU32};

use crate::RepoItem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to find item '{id:?}'.")]
    ItemNotFound { id: u32 },
}

pub struct InMemoryRepository<T: Clone> {
    items: Vec<RepoItem<T>>,
    counter: AtomicU32,
    // using a hashmap of orders is hard here because of how we return by value a non-cloneable Order
    // to make things easier, we'll keep a hashset of ids
    ids: HashSet<u32>,
}

impl<T: Clone> InMemoryRepository<T> {
    pub fn new() -> InMemoryRepository<T> {
        InMemoryRepository {
            counter: AtomicU32::new(1),
            items: Vec::new(),
            ids: HashSet::new(),
        }
    }

    pub fn items(&self) -> &Vec<RepoItem<T>> {
        &self.items
    }

    pub fn get_all(&self) -> Result<Vec<RepoItem<T>>, Error> {
        Ok(self.items.clone())
    }

    pub fn get(&self, id: u32) -> Result<RepoItem<T>, Error> {
        match self.ids.contains(&id) {
            true => self
                .items
                .iter()
                .find(|o| id == o.id())
                .cloned()
                .ok_or(Error::ItemNotFound { id }),
            false => Err(Error::ItemNotFound { id }),
        }
    }

    pub fn create(&mut self, item: T) -> Result<RepoItem<T>, Error> {
        let item = RepoItem(
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            item,
        );
        let result = item.clone();

        self.ids.insert(item.id());
        self.items.push(item);
        Ok(result)
    }

    pub fn remove(&mut self, item: RepoItem<T>) -> Result<RepoItem<T>, Error> {
        if self.ids.remove(&item.id()) {
            self.items.retain(|o| o.id() != item.id());
            Ok(item)
        } else {
            Err(Error::ItemNotFound { id: item.id() })
        }
    }

    pub fn update(&mut self, item: RepoItem<T>) -> Result<RepoItem<T>, Error> {
        let id = item.id();
        self.items
            .iter_mut()
            .find(|o| o.id() == item.id())
            .as_mut()
            // also, we do a copy here because either this map or the next will do a move,
            // and we can't do it in the next one because the first move will prevent us from copying
            .map(|o| **o = item.clone())
            .and(Some(item.clone()))
            .ok_or(Error::ItemNotFound { id })
    }
}

impl<T: Clone> Default for InMemoryRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}
