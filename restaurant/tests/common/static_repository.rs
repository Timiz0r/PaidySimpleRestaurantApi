use restaurant::{
    layout::{self, Table},
    menu::{self, MenuItem},
};

use super::Collection;

pub(crate) struct StaticRepository {
    pub menu: Collection<MenuItem>,
    pub tables: Collection<Table>,
}

impl menu::Repository for StaticRepository {
    async fn get_all(&self) -> menu::RepoResult<Vec<MenuItem>> {
        Ok(self.menu.clone().into())
    }

    async fn get(&self, _id: u32) -> menu::RepoResult<MenuItem> {
        unimplemented!()
    }

    async fn create(&mut self, _item: MenuItem) -> menu::RepoResult<MenuItem> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: MenuItem) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: MenuItem) -> menu::RepoResult<MenuItem> {
        unimplemented!()
    }
}

impl layout::TableRepository for StaticRepository {
    async fn get_all(&self) -> layout::RepoResult<Vec<Table>> {
        Ok(self.tables.clone().into())
    }

    async fn create(&mut self, _item: Table) -> layout::RepoResult<Table> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: Table) -> layout::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: Table) -> layout::RepoResult<Table> {
        unimplemented!()
    }
}

impl Clone for Collection<MenuItem> {
    fn clone(&self) -> Self {
        Collection(
            self.iter()
                .map(|i| MenuItem {
                    id: i.id,
                    name: i.name.clone(),
                    cook_time: i.cook_time,
                })
                .collect(),
        )
    }
}

impl Clone for Collection<Table> {
    fn clone(&self) -> Self {
        Collection(self.iter().map(|i| Table { id: i.id }).collect())
    }
}
