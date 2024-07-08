use std::sync::{Arc, RwLock};

use std::{
    clone::Clone, collections::HashSet, hash::Hash, result::Result, sync::atomic::AtomicU32,
};

use crate::{layout, menu, order, RepoItem};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<I> {
    #[error("Unable to find item '{id:?}'.")]
    ItemNotFound { id: I },
}

pub struct InMemoryRepository<T: Clone, I: Copy + Serialize> {
    items: Vec<RepoItem<T, I>>,
    idgen: Box<dyn IdGenerator<I> + Send + Sync>,
    // using a hashmap of orders is hard here because of how we return by value a non-cloneable Order
    // to make things easier, we'll keep a hashset of ids
    ids: HashSet<I>,
}

impl<T: Clone, I: Clone + Copy + PartialEq + Eq + Hash + Serialize> InMemoryRepository<T, I> {
    pub fn items(&self) -> &Vec<RepoItem<T, I>> {
        &self.items
    }

    pub fn get_all(&self) -> Result<Vec<RepoItem<T, I>>, Error<I>> {
        Ok(self.items.clone())
    }

    pub fn get(&self, id: I) -> Result<RepoItem<T, I>, Error<I>> {
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

    pub fn create(&mut self, item: T) -> Result<RepoItem<T, I>, Error<I>> {
        let item = RepoItem::<T, I>::new(self.idgen.get(), item);
        let result = item.clone();

        self.ids.insert(item.id());
        self.items.push(item);
        Ok(result)
    }

    pub fn remove(&mut self, id: I) -> Result<RepoItem<T, I>, Error<I>> {
        if self.ids.remove(&id) {
            let removed = self.items.iter().find(|m| m.id == id).unwrap().clone();
            self.items.retain(|o| o.id() != id);
            Ok(removed)
        } else {
            Err(Error::ItemNotFound { id })
        }
    }

    pub fn update(&mut self, item: RepoItem<T, I>) -> Result<RepoItem<T, I>, Error<I>> {
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

pub trait IdGenerator<I: Copy> {
    fn get(&self) -> I;
}

struct IdGeneratorImpl {
    table: AtomicU32,
    menu: AtomicU32,
    order: AtomicU32,
}

impl IdGenerator<layout::TableId> for IdGeneratorImpl {
    fn get(&self) -> layout::TableId {
        layout::TableId(
            self.table
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}

impl IdGenerator<menu::Id> for IdGeneratorImpl {
    fn get(&self) -> menu::Id {
        menu::Id(self.menu.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

impl IdGenerator<order::Id> for IdGeneratorImpl {
    fn get(&self) -> order::Id {
        order::Id(
            self.order
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}

impl Default for IdGeneratorImpl {
    fn default() -> Self {
        Self {
            table: AtomicU32::new(1),
            menu: AtomicU32::new(1),
            order: AtomicU32::new(1),
        }
    }
}

impl<T: Clone> Default for InMemoryRepository<T, order::Id> {
    fn default() -> Self {
        InMemoryRepository {
            idgen: Box::new(IdGeneratorImpl::default()),
            items: Vec::new(),
            ids: HashSet::new(),
        }
    }
}

impl<T: Clone> Default for InMemoryRepository<T, menu::Id> {
    fn default() -> Self {
        InMemoryRepository {
            idgen: Box::new(IdGeneratorImpl::default()),
            items: Vec::new(),
            ids: HashSet::new(),
        }
    }
}

impl<T: Clone> Default for InMemoryRepository<T, layout::TableId> {
    fn default() -> Self {
        InMemoryRepository {
            idgen: Box::new(IdGeneratorImpl::default()),
            items: Vec::new(),
            ids: HashSet::new(),
        }
    }
}

type Table<T, I> = Arc<RwLock<InMemoryRepository<T, I>>>;
#[derive(Clone, Default)]
pub struct Database {
    menu: Table<menu::Item, menu::Id>,
    tables: Table<layout::Table, layout::TableId>,
    orders: Table<order::Order, order::Id>,
}

impl Database {
    pub fn new(
        menu: Vec<menu::RepoItem>,
        tables: Vec<layout::RepoTable>,
        orders: Vec<order::RepoOrder>,
    ) -> Database {
        Database {
            menu: Arc::new(RwLock::new(InMemoryRepository {
                items: menu,
                ..Default::default()
            })),
            tables: Arc::new(RwLock::new(InMemoryRepository {
                items: tables,
                ..Default::default()
            })),
            orders: Arc::new(RwLock::new(InMemoryRepository {
                items: orders,
                ..Default::default()
            })),
        }
    }
}

impl menu::Repository for Database {
    async fn get_all(&self) -> menu::RepoResult<Vec<menu::RepoItem>> {
        Ok(self.menu.read().unwrap().items().clone())
    }

    async fn get(&self, id: menu::Id) -> menu::RepoResult<menu::RepoItem> {
        self.menu
            .read()
            .unwrap()
            .get(id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create(&mut self, _id: menu::Item) -> menu::RepoResult<menu::RepoItem> {
        unimplemented!()
    }

    async fn remove(&mut self, _id: menu::Id) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: menu::RepoItem) -> menu::RepoResult<()> {
        unimplemented!()
    }
}

impl layout::TableRepository for Database {
    async fn get_all(&self) -> layout::RepoResult<Vec<layout::RepoTable>> {
        Ok(self.tables.read().unwrap().items().clone())
    }

    async fn get(&self, id: layout::TableId) -> menu::RepoResult<layout::RepoTable> {
        self.tables
            .read()
            .unwrap()
            .get(id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create(&mut self, _id: layout::Table) -> layout::RepoResult<layout::RepoTable> {
        unimplemented!()
    }

    async fn remove(&mut self, _id: layout::TableId) -> layout::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: layout::RepoTable) -> layout::RepoResult<()> {
        unimplemented!()
    }
}

impl order::Repository for Database {
    async fn get_all(&self) -> order::RepoResult<Vec<order::RepoOrder>> {
        self.orders
            .read()
            .unwrap()
            .get_all()
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_table(
        &self,
        table_id: layout::TableId,
    ) -> order::RepoResult<Vec<order::RepoOrder>> {
        let results = self
            .orders
            .read()
            .unwrap()
            .items()
            .iter()
            .filter(|o| o.table_id == table_id)
            .cloned()
            .collect::<Vec<order::RepoOrder>>();
        if results.is_empty() {
            Err(anyhow::anyhow!(
                "No orders found for table '{:?}'.",
                table_id
            ))
        } else {
            Ok(results)
        }
    }

    async fn get(&self, id: order::Id) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .read()
            .unwrap()
            .get(id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn create(&mut self, item: order::Order) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .create(item)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn remove(&mut self, id: order::Id) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .remove(id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn update(&mut self, item: order::RepoOrder) -> order::RepoResult<order::RepoOrder> {
        self.orders
            .write()
            .unwrap()
            .update(item)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
