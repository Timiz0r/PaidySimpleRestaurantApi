use restaurant::{
    layout::{self, RepoTable, Table},
    menu::{self, MenuItem, RepoMenuItem},
    RepoItem,
};

use super::Collection;

pub(crate) struct StaticRepository {
    pub menu: Collection<RepoMenuItem>,
    pub tables: Collection<RepoTable>,
}

impl menu::Repository for StaticRepository {
    async fn get_all(&self) -> menu::RepoResult<Vec<RepoMenuItem>> {
        Ok(self.menu.clone().into())
    }

    async fn get(&self, _id: u32) -> menu::RepoResult<RepoMenuItem> {
        unimplemented!()
    }

    async fn create(&mut self, _item: MenuItem) -> menu::RepoResult<RepoMenuItem> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: RepoMenuItem) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: RepoMenuItem) -> menu::RepoResult<()> {
        unimplemented!()
    }
}

impl layout::TableRepository for StaticRepository {
    async fn get_all(&self) -> layout::RepoResult<Vec<RepoTable>> {
        Ok(self.tables.clone().into())
    }

    async fn create(&mut self, _item: Table) -> layout::RepoResult<RepoTable> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: RepoTable) -> layout::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: RepoTable) -> layout::RepoResult<()> {
        unimplemented!()
    }
}

impl Clone for Collection<RepoMenuItem> {
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

impl Clone for Collection<RepoTable> {
    fn clone(&self) -> Self {
        Collection(self.iter().map(|i| RepoItem(i.id(), Table {})).collect())
    }
}
