use chrono::{TimeDelta, Utc};
use common::{ComparableOrder, OrderRepository, StaticRepository};
use futures::executor::LocalPool;
use pretty_assertions::assert_eq;
use restaurant::layout;
use restaurant::menu;
use restaurant::{
    order::{self, OrderingError},
    RepoItem,
};

mod common;

#[test]
fn place_orders() -> Result<(), OrderingError> {
    // we don't actually need StaticRepository for ordering tests
    // but using it to roughly illustrate its usage
    let statics = StaticRepository {
        menu: vec![RepoItem(
            1,
            menu::Item {
                name: "Pasta".to_string(),
                cook_time: TimeDelta::minutes(5),
            },
        )],
        tables: vec![RepoItem(1, layout::Table {})],
    };
    let mut repo = OrderRepository::new();

    futures::executor::block_on(order::place(
        &mut repo,
        statics.tables[0].clone(),
        statics.menu[0].clone(),
        3,
    ))?;

    assert_eq!(
        &[ComparableOrder(RepoItem(
            1,
            order::Order {
                table: RepoItem(1, layout::Table {}),
                menu_item: RepoItem(
                    1,
                    menu::Item {
                        name: "Pasta".to_string(),
                        cook_time: TimeDelta::minutes(5),
                    }
                ),
                time_placed: Utc::now(),
                quantity: 3
            }
        )),][..],
        repo.orders().as_slice()
    );

    Ok(())
}

#[test]
fn change_order_quantity() -> Result<(), OrderingError> {
    let mut pool = LocalPool::new();
    pool.run_until(async {
        let table1 = RepoItem(1, layout::Table {});
        let table2 = RepoItem(3, layout::Table {});
        let pasta = RepoItem(
            1,
            menu::Item {
                name: "Pasta".to_string(),
                cook_time: TimeDelta::minutes(5),
            },
        );
        let sandwich = RepoItem(
            1,
            menu::Item {
                name: "Sandwich".to_string(),
                cook_time: TimeDelta::minutes(5),
            },
        );
        let mut repo = OrderRepository::new();

        async fn place_order(
            repo: &mut OrderRepository,
            table: &layout::RepoTable,
            item: &menu::RepoItem,
            quantity: u32,
        ) -> order::Result<(u32, order::RepoOrder)> {
            order::place(repo, table.clone(), item.clone(), quantity)
                .await
                .map(|o| (o.id(), o))
        }

        let (id1, order1) = place_order(&mut repo, &table1, &pasta, 3).await?;
        let (_, order2) = place_order(&mut repo, &table1, &sandwich, 2).await?;
        let (id3, order3) = place_order(&mut repo, &table2, &sandwich, 5).await?;

        order::set_quantity(&mut repo, order1, 1).await?;
        order::set_quantity(&mut repo, order2, 0).await?;
        order::set_quantity(&mut repo, order3, 7).await?;

        let mut orders1 = order::get(&mut repo, table1.clone()).await?;
        orders1.sort_by_key(|a| a.id());
        let mut orders2 = order::get(&mut repo, table2.clone()).await?;
        orders2.sort_by_key(|a| a.id());

        assert_eq!(
            // setting to zero removes, so only two
            &[ComparableOrder(RepoItem(
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
            &[ComparableOrder(RepoItem(
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

        Ok(())
    })
}
