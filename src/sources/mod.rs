use std::fmt::Debug;

use mlua::Lua;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString};

use crate::{entry::Entry, injector::FinderFn};

pub mod diagnostics;
pub mod files;
pub mod git;
mod lua_function;
pub mod lua_tables;

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Default, PartialEq, EnumString, Display, EnumIs, Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    Rust,
    Lua,
}

pub trait Populator<T, U, V>
where
    T: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    V: Entry,
{
    fn name(&self) -> String;
    fn kind(&self) -> SourceKind;
    fn update_config(&mut self, config: U);

    fn build_injector(&self, lua: Option<&Lua>) -> FinderFn<V>;
}
