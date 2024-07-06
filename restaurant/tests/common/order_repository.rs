use restaurant::{
    layout,
    mem_repo::InMemoryRepository,
    order::{self, Order, RepoOrder},
};

pub(crate) struct OrderRepository {
    repo: InMemoryRepository<Order>,
}

impl OrderRepository {
    pub fn new() -> OrderRepository {
        OrderRepository {
            repo: InMemoryRepository::new(),
        }
    }

    // of course there's get_all, but this saves a copy when inspecting for testing
    pub fn orders(&self) -> &Vec<RepoOrder> {
        self.repo.items()
    }
}

impl order::Repository for OrderRepository {
    async fn get_all(&self) -> order::RepoResult<Vec<RepoOrder>> {
        self.repo.get_all().await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_table(&self, table: layout::RepoTable) -> order::RepoResult<Vec<RepoOrder>> {
        let results = self
            .repo
            .items()
            .iter()
            .filter(|o| o.table.id() == table.id())
            .cloned()
            .collect::<Vec<RepoOrder>>();
        if results.is_empty() {
            Err(anyhow::anyhow!(
                "No orders found for table '{}'.",
                table.id()
            ))
        } else {
            Ok(results)
        }
    }

    async fn get(&self, id: u32) -> order::RepoResult<RepoOrder> {
        self.repo.get(id).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn create(&mut self, item: Order) -> order::RepoResult<RepoOrder> {
        self.repo.create(item).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn remove(&mut self, item: RepoOrder) -> order::RepoResult<RepoOrder> {
        self.repo.remove(item).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn update(&mut self, item: RepoOrder) -> order::RepoResult<RepoOrder> {
        self.repo.update(item).await.map_err(|e| anyhow::anyhow!(e))
    }
}
