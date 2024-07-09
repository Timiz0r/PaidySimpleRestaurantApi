use chrono::Utc;
use common::ComparableOrder;
use futures::executor::LocalPool;
use pretty_assertions::assert_eq;
use restaurant::layout::{self};
use restaurant::memdb::Database;
use restaurant::menu;
use restaurant::{
    order::{self, OrderingError},
    RepoItem,
};

mod common;

#[test]
fn place_orders() -> Result<(), OrderingError> {
    let mut pool = LocalPool::new();
    pool.run_until(async {
        // we don't actually need to use db for menu and layout
        // but using it to roughly illustrate its usage
        let mut db = Database::new(
            vec![RepoItem::new(
                1.into(),
                menu::Item {
                    name: "Pasta".to_string(),
                    cook_time: menu::Minutes(5),
                },
            )],
            vec![RepoItem::new(1.into(), layout::Table {})],
            vec![],
        );

        let table = layout::TableRepository::get(&db, layout::TableId(1))
            .await
            .expect("Table 1 should exist");
        let item = menu::Repository::get(&db, menu::Id(1))
            .await
            .expect("Item 1 should exist");
        order::place(&mut db, table.clone(), item.clone(), 3).await?;

        assert_eq!(
            &[ComparableOrder(RepoItem::new(
                1.into(),
                order::Order {
                    table: table.clone(),
                    menu_item: item.clone(),
                    time_placed: Utc::now(),
                    quantity: 3
                }
            )),][..],
            order::Repository::get_all(&db)
                .await
                .expect("Getting all orders should not fail.")
                .as_slice()
        );

        Ok(())
    })
}

#[test]
fn change_order_quantity() -> Result<(), OrderingError> {
    let mut pool = LocalPool::new();
    pool.run_until(async {
        let table1 = RepoItem::new(1.into(), layout::Table {});
        let table2 = RepoItem::new(3.into(), layout::Table {});
        let pasta = RepoItem::new(
            1.into(),
            menu::Item {
                name: "Pasta".to_string(),
                cook_time: menu::Minutes(5),
            },
        );
        let sandwich = RepoItem::new(
            1.into(),
            menu::Item {
                name: "Sandwich".to_string(),
                cook_time: menu::Minutes(5),
            },
        );
        let mut db = Database::default();

        async fn place_order(
            db: &mut Database,
            table: &layout::RepoTable,
            item: &menu::RepoItem,
            quantity: u32,
        ) -> order::Result<(order::Id, order::RepoOrder)> {
            order::place(db, table.clone(), item.clone(), quantity)
                .await
                .map(|o| (o.id(), o))
        }

        let (id1, order1) = place_order(&mut db, &table1, &pasta, 3).await?;
        let (id2, order2) = place_order(&mut db, &table1, &sandwich, 2).await?;
        let (id3, order3) = place_order(&mut db, &table2, &sandwich, 5).await?;

        order::set_quantity(&mut db, order1.id(), 1).await?;
        let zero_quantity_order = order::set_quantity(&mut db, order2.id(), 0).await?;
        order::set_quantity(&mut db, order3.id(), 7).await?;

        let mut orders1 = order::get_table(&db, table1.id()).await?;
        orders1.sort_by_key(|a| a.id());
        let mut orders2 = order::get_table(&db, table2.id()).await?;
        orders2.sort_by_key(|a| a.id());

        assert_eq!(
            // setting to zero removes, so only two
            &[ComparableOrder(RepoItem::new(
                id1,
                order::Order {
                    table: table1.clone(),
                    menu_item: pasta.clone(),
                    quantity: 1,
                    time_placed: Utc::now()
                }
            ))][..],
            orders1.as_slice()
        );

        assert_eq!(
            // setting to zero removes, so only two
            &[ComparableOrder(RepoItem::new(
                id3,
                order::Order {
                    table: table2.clone(),
                    menu_item: sandwich.clone(),
                    quantity: 7,
                    time_placed: Utc::now()
                }
            ))][..],
            orders2.as_slice()
        );

        assert_eq!(
            ComparableOrder(RepoItem::new(
                id2,
                order::Order {
                    table: table1.clone(),
                    menu_item: sandwich.clone(),
                    quantity: 0,
                    time_placed: Utc::now()
                }
            )),
            zero_quantity_order
        );

        Ok(())
    })
}

#[test]
fn cancel_order() -> Result<(), OrderingError> {
    let mut pool = LocalPool::new();
    pool.run_until(async {
        let table = RepoItem::new(1.into(), layout::Table {});
        let item = RepoItem::new(
            1.into(),
            menu::Item {
                name: "Pasta".to_string(),
                cook_time: menu::Minutes(5),
            },
        );
        let mut db = Database::default();
        let order = order::place(&mut db, table.clone(), item.clone(), 12).await?;
        order::cancel(&mut db, order.id()).await?;

        if let Ok(orders) = order::get_table(&db, table.id()).await {
            assert!(orders.is_empty(), "Orders found.")
        };

        Ok(())
    })
}
