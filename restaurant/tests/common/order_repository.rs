use std::{collections::HashSet, sync::atomic::AtomicU32};

use restaurant::ordering::{self, Order};
use thiserror::Error;

use super::Collection;

#[derive(Error, Debug)]
enum Error {
    #[error("Unable to find order '{id:?}'.")]
    OrderNotFound { id: Option<u32> },
}

pub(crate) struct OrderRepository<'a> {
    orders: Collection<Order<'a>>,
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

    pub fn orders(&self) -> &Vec<Order<'a>> {
        &self.orders
    }
}

impl<'a> ordering::Repository<'a> for OrderRepository<'a> {
    async fn get_all(&self) -> ordering::RepoResult<Vec<Order>> {
        Ok(self.orders.clone().into())
    }

    async fn get(&self, id: u32) -> ordering::RepoResult<Order> {
        match self.ids.contains(&id) {
            true => self
                .orders
                .clone()
                .iter()
                .find(|o| o.id.is_some_and(|i| i == id))
                .map(|o| Order { ..*o })
                .ok_or_else(|| anyhow::anyhow!(Error::OrderNotFound { id: Some(id) })),
            false => Err(anyhow::anyhow!(Error::OrderNotFound { id: Some(id) })),
        }
    }

    async fn create(&mut self, item: Order<'a>, quantity: u32) -> ordering::RepoResult<()> {
        (0..quantity).for_each(|_| {
            self.orders.push(Order {
                id: Some(
                    self.counter
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                ),
                ..item
            })
        });
        Ok(())
    }

    async fn remove(&mut self, item: Order<'a>) -> ordering::RepoResult<()> {
        match item.id {
            Some(id) if self.ids.remove(&id) => Ok(()),
            _ => Err(anyhow::anyhow!(Error::OrderNotFound { id: item.id })),
        }
    }

    async fn update(&mut self, item: Order<'a>) -> ordering::RepoResult<()> {
        match item.id {
            Some(id) => self
                .orders
                .iter_mut()
                .find(|o| o.id.filter(|cur| *cur == id).is_some())
                .as_mut()
                .map(|o| **o = item)
                .ok_or_else(|| anyhow::anyhow!(Error::OrderNotFound { id: Some(id) })),
            _ => Err(anyhow::anyhow!(Error::OrderNotFound { id: item.id })),
        }
    }
}

impl Clone for Collection<Order<'_>> {
    fn clone(&self) -> Self {
        Collection(
            self.iter()
                .map(|i| Order {
                    id: i.id,
                    table: i.table,
                    menu_item: i.menu_item,
                    time_placed: i.time_placed,
                })
                .collect(),
        )
    }
}
