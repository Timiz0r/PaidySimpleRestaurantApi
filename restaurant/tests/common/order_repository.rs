use std::{collections::HashSet, sync::atomic::AtomicU32};

use restaurant::{
    layout,
    order::{self, Order, RepoOrder},
    RepoItem,
};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Unable to find order '{id:?}'.")]
    OrderNotFound { id: u32 },
}

pub(crate) struct OrderRepository {
    orders: Vec<RepoOrder>,
    counter: AtomicU32,
    // using a hashmap of orders is hard here because of how we return by value a non-cloneable Order
    // to make things easier, we'll keep a hashset of ids
    ids: HashSet<u32>,
}

impl OrderRepository {
    pub fn new() -> OrderRepository {
        OrderRepository {
            counter: AtomicU32::new(1),
            orders: Vec::new(),
            ids: HashSet::new(),
        }
    }

    // of course there's get_all, but this saves a copy when inspecting for testing
    pub fn orders(&self) -> &Vec<RepoOrder> {
        &self.orders
    }
}

impl order::Repository for OrderRepository {
    async fn get_all(&self) -> order::RepoResult<Vec<RepoOrder>> {
        Ok(self.orders.clone())
    }

    async fn get_table(&self, table: layout::RepoTable) -> order::RepoResult<Vec<RepoOrder>> {
        Ok(self
            .orders
            .iter()
            .filter(|o| o.table.id() == table.id())
            .cloned()
            .collect())
    }

    async fn get(&self, id: u32) -> order::RepoResult<RepoOrder> {
        match self.ids.contains(&id) {
            true => self
                .orders
                .iter()
                .find(|o| id == o.id())
                .cloned()
                .ok_or_else(|| anyhow::anyhow!(Error::OrderNotFound { id })),
            false => Err(anyhow::anyhow!(Error::OrderNotFound { id })),
        }
    }

    async fn create(&mut self, item: Order) -> order::RepoResult<RepoOrder> {
        let item = RepoItem(
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            item,
        );
        let result = item.clone();

        self.ids.insert(item.id());
        self.orders.push(item);
        Ok(result)
    }

    async fn remove(&mut self, item: RepoOrder) -> order::RepoResult<RepoOrder> {
        if self.ids.remove(&item.id()) {
            self.orders.retain(|o| o.id() != item.id());
            Ok(item)
        } else {
            Err(anyhow::anyhow!(Error::OrderNotFound { id: item.id() }))
        }
    }

    async fn update(&mut self, item: RepoOrder) -> order::RepoResult<RepoOrder> {
        let id = item.id();
        self.orders
            .iter_mut()
            .find(|o| o.id() == item.id())
            .as_mut()
            // also, we do a copy here because either this map or the next will do a move,
            // and we can't do it in the next one because the first move will prevent us from copying
            .map(|o| **o = item.clone())
            .and(Some(item.clone()))
            .ok_or_else(|| anyhow::anyhow!(Error::OrderNotFound { id }))
    }
}
