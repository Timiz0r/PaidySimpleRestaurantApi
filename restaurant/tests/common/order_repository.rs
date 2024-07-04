use std::{collections::HashSet, sync::atomic::AtomicU32};

use restaurant::{
    ordering::{self, Order},
    RepoItem,
};
use thiserror::Error;

use super::Collection;

#[derive(Error, Debug)]
enum Error {
    #[error("Unable to find order '{id:?}'.")]
    OrderNotFound { id: u32 },
}

pub(crate) struct OrderRepository<'a> {
    orders: Collection<RepoItem<Order<'a>>>,
    counter: AtomicU32,
    // using a hashmap of orders is hard here because of how we return by value a non-cloneable Order
    // to make things easier, we'll keep a hashset of ids
    ids: HashSet<u32>,
}

impl<'a> OrderRepository<'a> {
    pub fn new() -> OrderRepository<'a> {
        OrderRepository {
            counter: AtomicU32::new(1),
            orders: Collection(Vec::new()),
            ids: HashSet::new(),
        }
    }

    // of course there's get_all, but this saves a copy when inspecting for testing
    pub fn orders(&self) -> &Vec<RepoItem<Order<'a>>> {
        &self.orders
    }
}

impl<'a> ordering::Repository<'a> for OrderRepository<'a> {
    async fn get_all(&self) -> ordering::RepoResult<Vec<RepoItem<Order>>> {
        Ok(self.orders.clone().into())
    }

    async fn get(&self, id: u32) -> ordering::RepoResult<RepoItem<Order>> {
        match self.ids.contains(&id) {
            true => self
                .orders
                .clone()
                .iter()
                .find(|o| id == o.id())
                .map(|o| RepoItem(o.id(), Order { ..*o.item() }))
                .ok_or_else(|| anyhow::anyhow!(Error::OrderNotFound { id })),
            false => Err(anyhow::anyhow!(Error::OrderNotFound { id })),
        }
    }

    async fn create(&mut self, item: Order<'a>, quantity: u32) -> ordering::RepoResult<()> {
        (0..quantity).for_each(|_| {
            self.orders.push(RepoItem(
                self.counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                Order { ..item },
            ))
        });
        Ok(())
    }

    async fn remove(&mut self, item: RepoItem<Order<'a>>) -> ordering::RepoResult<()> {
        self.ids
            .remove(&item.id())
            .then_some(())
            .ok_or(anyhow::anyhow!(Error::OrderNotFound { id: item.id() }))
    }

    async fn update(&mut self, item: RepoItem<Order<'a>>) -> ordering::RepoResult<()> {
        // if we need to return an error, we need this copied because we move item in the below map
        let id = item.id();

        self.orders
            .iter_mut()
            .find(|o| o.id() == id)
            .as_mut()
            .map(|o| **o = item)
            .ok_or(anyhow::anyhow!(Error::OrderNotFound { id }))
    }
}

impl Clone for Collection<RepoItem<Order<'_>>> {
    fn clone(&self) -> Self {
        Collection(
            self.iter()
                .map(|i| {
                    RepoItem(
                        i.id(),
                        Order {
                            table: i.table,
                            menu_item: i.menu_item,
                            time_placed: i.time_placed,
                        },
                    )
                })
                .collect(),
        )
    }
}
