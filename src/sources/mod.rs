use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{entry::Entry, injector::FinderFn};

pub mod diagnostics;
pub mod files;
pub mod git;
pub mod lua_tables;

pub trait Populator<T, U, V>
where
    T: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    V: Entry,
{
    fn name(&self) -> String;
    fn update_config(&mut self, config: U);

    fn build_injector(&self) -> FinderFn<V>;
}
