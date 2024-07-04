use chrono::{TimeDelta, Utc};
use common::{ComparableOrder, OrderRepository, StaticRepository};
use pretty_assertions::assert_eq;
use restaurant::{
    layout::Table,
    menu::MenuItem,
    ordering::{self, Order, OrderingError},
    RepoItem,
};

mod common;

#[test]
fn place_orders() -> Result<(), OrderingError> {
    // we don't actually need StaticRepository for this test
    // but using it to roughly illustrate its usage
    let statics = StaticRepository {
        menu: vec![RepoItem(
            1,
            MenuItem {
                name: "Pasta".to_string(),
                cook_time: TimeDelta::minutes(5),
            },
        )]
        .into(),
        tables: vec![RepoItem(1, Table {})].into(),
    };
    let mut repo = OrderRepository::new();

    futures::executor::block_on(ordering::place_order(
        &mut repo,
        &statics.tables[0],
        &statics.menu[0],
        3,
    ))?;

    let expected_table = RepoItem(1, Table {});
    let expected_item = RepoItem(
        1,
        MenuItem {
            name: "Pasta".to_string(),
            cook_time: TimeDelta::minutes(5),
        },
    );
    assert_eq!(
        &[
            ComparableOrder(RepoItem(
                1,
                Order {
                    table: &expected_table,
                    menu_item: &expected_item,
                    time_placed: Utc::now()
                }
            )),
            ComparableOrder(RepoItem(
                2,
                Order {
                    table: &expected_table,
                    menu_item: &expected_item,
                    time_placed: Utc::now()
                }
            )),
            ComparableOrder(RepoItem(
                3,
                Order {
                    table: &expected_table,
                    menu_item: &expected_item,
                    time_placed: Utc::now()
                }
            )),
        ][..],
        repo.orders().as_slice()
    );

    Ok(())
}
