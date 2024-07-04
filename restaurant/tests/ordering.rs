use chrono::{TimeDelta, Utc};
use common::{ComparableOrder, OrderRepository, StaticRepository};
use pretty_assertions::assert_eq;
use restaurant::{
    layout::Table,
    menu::MenuItem,
    ordering::{self, Order, OrderingError},
};

mod common;

#[test]
fn place_orders() -> Result<(), OrderingError> {
    // we don't actually need StaticRepository for this test
    // but using it to roughly illustrate its usage
    let statics = StaticRepository {
        menu: vec![MenuItem {
            id: Some(1),
            name: "Pasta".to_string(),
            cook_time: TimeDelta::minutes(5),
        }]
        .into(),
        tables: vec![Table { id: Some(1) }].into(),
    };
    let mut repo = OrderRepository::new();

    futures::executor::block_on(ordering::place_order(
        &mut repo,
        &statics.tables[0],
        &statics.menu[0],
        3,
    ))?;

    assert_eq!(
        &[
            ComparableOrder(Order {
                id: Some(1),
                table: &Table { id: Some(1) },
                menu_item: &MenuItem {
                    id: Some(1),
                    name: "Pasta".to_string(),
                    cook_time: TimeDelta::minutes(5)
                },
                time_placed: Utc::now()
            }),
            ComparableOrder(Order {
                id: Some(2),
                table: &Table { id: Some(1) },
                menu_item: &MenuItem {
                    id: Some(1),
                    name: "Pasta".to_string(),
                    cook_time: TimeDelta::minutes(5)
                },
                time_placed: Utc::now()
            }),
            ComparableOrder(Order {
                id: Some(3),
                table: &Table { id: Some(1) },
                menu_item: &MenuItem {
                    id: Some(1),
                    name: "Pasta".to_string(),
                    cook_time: TimeDelta::minutes(5)
                },
                time_placed: Utc::now()
            }),
        ][..],
        repo.orders().as_slice()
    );

    Ok(())
}
