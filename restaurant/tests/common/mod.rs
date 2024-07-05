mod order_repository;
mod static_repository;

use std::fmt::{self, Debug};

pub(crate) use order_repository::*;
use restaurant::order;
// these arent currently used, but implemented it to see what the complete impl would look like
pub(crate) use static_repository::*;

pub(crate) struct ComparableOrder(pub order::RepoOrder);

// would be obnoxious to maintain all the fields, so just trust id
// not currently implementing PartialEq for these types because they aren't meant to be =='d, but we could do it anyway
impl PartialEq<restaurant::RepoItem<order::Order>> for ComparableOrder {
    fn eq(&self, other: &order::RepoOrder) -> bool {
        self.0.id() == other.id()
            && self.0.table.id() == other.table.id()
            && self.0.menu_item.id() == other.menu_item.id()
        // TODO: see if we can make a clock a driven port
    }
}

impl Debug for ComparableOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(&self.0.item(), f)
    }
}
