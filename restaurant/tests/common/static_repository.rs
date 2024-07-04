use restaurant::{
    layout::{self, Table},
    menu::{self, MenuItem},
    RepoItem,
};

use super::Collection;

pub(crate) struct StaticRepository {
    pub menu: Collection<RepoItem<MenuItem>>,
    pub tables: Collection<RepoItem<Table>>,
}

impl menu::Repository for StaticRepository {
    async fn get_all(&self) -> menu::RepoResult<Vec<RepoItem<MenuItem>>> {
        Ok(self.menu.clone().into())
    }

    async fn get(&self, _id: u32) -> menu::RepoResult<RepoItem<MenuItem>> {
        unimplemented!()
    }

    async fn create(&mut self, _item: MenuItem) -> menu::RepoResult<RepoItem<MenuItem>> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: RepoItem<MenuItem>) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: RepoItem<MenuItem>) -> menu::RepoResult<()> {
        unimplemented!()
    }
}

impl layout::TableRepository for StaticRepository {
    async fn get_all(&self) -> layout::RepoResult<Vec<RepoItem<Table>>> {
        Ok(self.tables.clone().into())
    }

    async fn create(&mut self, _item: Table) -> layout::RepoResult<RepoItem<Table>> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: RepoItem<Table>) -> layout::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: RepoItem<Table>) -> layout::RepoResult<()> {
        unimplemented!()
    }
}

impl Clone for Collection<RepoItem<MenuItem>> {
    fn clone(&self) -> Self {
        Collection(
            self.iter()
                .map(|i| {
                    RepoItem(
                        i.id(),
                        MenuItem {
                            name: i.name.clone(),
                            cook_time: i.cook_time,
                        },
                    )
                })
                .collect(),
        )
    }
}

impl Clone for Collection<RepoItem<Table>> {
    fn clone(&self) -> Self {
        Collection(self.iter().map(|i| RepoItem(i.id(), Table {})).collect())
    }
}
