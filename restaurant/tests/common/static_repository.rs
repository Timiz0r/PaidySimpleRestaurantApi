use restaurant::{
    layout::{self, RepoTable, Table},
    menu::{self, Item, RepoItem},
};

pub(crate) struct StaticRepository {
    pub menu: Vec<RepoItem>,
    pub tables: Vec<RepoTable>,
}

impl menu::Repository for StaticRepository {
    async fn get_all(&self) -> menu::RepoResult<Vec<RepoItem>> {
        Ok(self.menu.clone())
    }

    async fn get(&self, _id: u32) -> menu::RepoResult<RepoItem> {
        unimplemented!()
    }

    async fn create(&mut self, _item: Item) -> menu::RepoResult<RepoItem> {
        unimplemented!()
    }

    async fn remove(&mut self, _item: RepoItem) -> menu::RepoResult<()> {
        unimplemented!()
    }

    async fn update(&mut self, _item: RepoItem) -> menu::RepoResult<()> {
        unimplemented!()
    }
}

impl layout::TableRepository for StaticRepository {
    async fn get_all(&self) -> layout::RepoResult<Vec<RepoTable>> {
        Ok(self.tables.clone())
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
