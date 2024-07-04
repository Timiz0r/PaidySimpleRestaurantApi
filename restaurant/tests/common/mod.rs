mod order_repository;
mod static_repository;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub(crate) use order_repository::*;
use restaurant::ordering::Order;
// these arent currently used, but implemented it to see what the complete impl would look like
pub(crate) use static_repository::*;

pub(crate) struct ComparableOrder<'a>(pub Order<'a>);

// would be obnoxious to maintain all the fields, so just trust id
// not currently implementing PartialEq for these types because they aren't meant to be =='d, but we could do it anyway
impl<'a> PartialEq<Order<'a>> for ComparableOrder<'a> {
    fn eq(&self, other: &Order<'a>) -> bool {
        self.0.id == other.id
            && self.0.table.id == other.table.id
            && self.0.menu_item.id == other.menu_item.id
        // TODO: see if we can make a clock a driven port
    }
}

impl<'a> Debug for ComparableOrder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

pub(crate) struct Collection<T>(Vec<T>);

impl<T> From<Collection<T>> for Vec<T> {
    fn from(val: Collection<T>) -> Self {
        val.0
    }
}

impl<T> From<Vec<T>> for Collection<T> {
    fn from(val: Vec<T>) -> Self {
        Collection(val)
    }
}

impl<T> Deref for Collection<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Collection<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
