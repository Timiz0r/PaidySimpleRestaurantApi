use anyhow::anyhow;
use chrono::{DateTime, Utc};
use futures::TryFutureExt;
use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Distribution, Standard,
    },
    seq::IteratorRandom,
    Rng,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

#[tokio::main(worker_threads = 16)]
async fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or("http://127.0.0.1:13982/".to_string());
    let client = reqwest::Client::new();

    let mut tasks: JoinSet<Result<(), anyhow::Error>> = JoinSet::new();
    (1..=100).for_each(|i| {
        let root = root.clone();
        let client = client.clone();
        tasks.spawn(async move {
            let simulator = OrderSimulator {
                root: Url::parse(&root).expect("Invalid root url"),
                table: i,
                client: &client,
            };
            loop {
                simulator.simulate_table().await?;
            }
        });
    });

    while let Some(task) = tasks.join_next().await {
        if let Err(joinerr) = task {
            println!("Join error: {:?}", joinerr);
        } else if let Ok(Err(err)) = task {
            println!("Simulation error: {:?}", err);
        } else {
            panic!("Somehow completed infinitely running task successfully.")
        }
    }
}

struct OrderSimulator<'a> {
    root: Url,
    table: u32,
    client: &'a Client,
}

impl<'a> OrderSimulator<'a> {
    async fn simulate_table(&self) -> Result<(), anyhow::Error> {
        let orders = self.get_orders().await?;
        assert!(
            orders.is_empty(),
            "Orders found for what should be a clear table {}",
            self.table
        );

        let order_count = self.gen_range(5..=10);
        let mut orders: Vec<u32> = Vec::with_capacity(order_count);
        for _ in 0..order_count {
            let order = CreateOrder {
                table_id: self.table,
                item_id: self.gen::<Menu>() as u32,
                quantity: self.gen_range(1..=10),
            };

            orders.push(self.create_order(order).await?.id);
        }
        assert_eq!(
            orders.len(),
            order_count,
            "Attempted to create {} order, got {}.",
            order_count,
            orders.len()
        );

        for _ in 0..self.gen_range(1..=5) {
            let order = self
                .choose(orders.iter())
                .expect("Somehow didnt choose a random order.");
            let quantity = self.gen_range(1..=20);
            let result = self.set_quantity(*order, quantity).await?;
            assert_eq!(
                quantity, result.quantity,
                "Set quantity to {}, got {}",
                quantity, result.quantity
            );

            if result.quantity == 0 {
                orders.retain(|id| *id != result.id);
            }
        }

        for _ in 0..self.gen_range(1..=7) {
            if let Some(order) = self.choose(orders.iter()) {
                let result = self.delete_order(*order).await?;
                orders.retain(|id| *id != result.id);
            }
        }

        let final_orders = self.clear_table().await?;
        assert_eq!(
            orders.len(),
            final_orders.len(),
            "Expected {} orders remaining, got {}.",
            orders.len(),
            final_orders.len()
        );

        let orders = self.get_orders().await?;
        assert_eq!(
            0,
            orders.len(),
            "Expected 0 orders remaining, got {}.",
            orders.len()
        );

        Ok(())
    }

    async fn get_orders(&self) -> Result<Vec<OrderDetails>, anyhow::Error> {
        self.client
            .get(
                self.root
                    .join(format!("/api/table/{}/orders", self.table).as_str())
                    .unwrap(),
            )
            .header("x-api-version", "v1")
            .send()
            .and_then(|r| async { r.error_for_status() })
            .and_then(|r| r.json())
            .map_err(|e| anyhow!(e))
            .await
    }

    async fn create_order(&self, order: CreateOrder) -> Result<OrderDetails, anyhow::Error> {
        self.client
            .post(self.root.join("/api/orders").unwrap())
            .header("x-api-version", "v1")
            .json(&order)
            .send()
            .and_then(|r| async { r.error_for_status() })
            .and_then(|r| r.json())
            .map_err(|e| anyhow!(e))
            .await
    }

    async fn set_quantity(&self, id: u32, quantity: u32) -> Result<OrderDetails, anyhow::Error> {
        self.client
            .post(
                self.root
                    .join(format!("/api/orders/{}/setquantity", id).as_str())
                    .unwrap(),
            )
            .header("x-api-version", "v1")
            .json(&SetOrderQuantity { quantity })
            .send()
            .and_then(|r| async { r.error_for_status() })
            .and_then(|r| r.json())
            .map_err(|e| anyhow!(e))
            .await
    }

    async fn delete_order(&self, id: u32) -> Result<OrderDetails, anyhow::Error> {
        self.client
            .delete(
                self.root
                    .join(format!("/api/orders/{}", id).as_str())
                    .unwrap(),
            )
            .header("x-api-version", "v1")
            .send()
            .and_then(|r| async { r.error_for_status() })
            .and_then(|r| r.json())
            .map_err(|e| anyhow!(e))
            .await
    }

    async fn clear_table(&self) -> Result<Vec<OrderDetails>, anyhow::Error> {
        self.client
            .post(
                self.root
                    .join(format!("/api/table/{}/clear", self.table).as_str())
                    .unwrap(),
            )
            .header("x-api-version", "v1")
            .send()
            .and_then(|r| async { r.error_for_status() })
            .and_then(|r| r.json())
            .map_err(|e| anyhow!(e))
            .await
    }

    // these are reimplemented here because Rng is not Send, which poses a problem considering all the awaiting being done
    fn gen_range<T: SampleUniform, R: SampleRange<T>>(&self, range: R) -> T {
        rand::thread_rng().gen_range(range)
    }
    fn gen<T>(&self) -> T
    where
        Standard: Distribution<T>,
    {
        rand::thread_rng().gen::<T>()
    }
    fn choose<T, I: Iterator<Item = T>>(&self, iter: I) -> Option<T> {
        iter.choose(&mut rand::thread_rng())
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OrderDetails {
    id: u32,
    table: TableDetails,
    menu_item: MenuItemDetails,
    time_placed: DateTime<Utc>,
    quantity: u32,
    estimated_minutes_remaining: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TableDetails {
    id: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MenuItemDetails {
    id: u32,
    name: String,
    cook_time: u32,
}

#[derive(Copy, Clone)]
enum Menu {
    Pasta = 1,
    Sandwich = 2,
    味噌カツ丼 = 3,
    和風パフェ = 4,
}

impl Distribution<Menu> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Menu {
        match rng.gen_range(1usize..=4) {
            1 => Menu::Pasta,
            2 => Menu::Sandwich,
            3 => Menu::味噌カツ丼,
            4 => Menu::和風パフェ,
            _ => panic!("Somehow hit a weird number"),
        }
    }
}

#[derive(Debug, Serialize)]
struct CreateOrder {
    table_id: u32,
    item_id: u32,
    quantity: u32,
}
#[derive(Debug, Serialize)]
struct SetOrderQuantity {
    quantity: u32,
}
