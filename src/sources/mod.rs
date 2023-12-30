use std::{fmt::Debug, str::FromStr};

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Deserializer, Serialize};
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

#[derive(Clone, Debug, PartialEq, EnumString, Display, EnumIs, Eq)]
pub enum Sources {
    #[strum(serialize = "builtin.files")]
    Files,
    #[strum(serialize = "builtin.git_status")]
    GitStatus,
    #[strum(serialize = "builtin.diagnostics")]
    Diagnostics,
    #[strum(default)]
    Custom(String),
}

impl Serialize for Sources {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Sources {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Sources::from_str(&s).expect("Strum should be defaulting here"))
    }
}

impl FromLua<'_> for Sources {
    fn from_lua(value: mlua::Value<'_>, lua: &'_ Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua<'_> for Sources {
    fn into_lua(self, lua: &'_ Lua) -> mlua::Result<mlua::Value<'_>> {
        self.to_string().into_lua(lua)
    }
}

pub trait Populator<T, U, V>
where
    T: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Debug + Clone + Serialize + for<'a> Deserialize<'a> + 'static,
    V: Entry,
{
    fn name(&self) -> Sources;
    fn kind(&self) -> SourceKind;
    fn update_config(&mut self, config: U);

    fn build_injector(&self, lua: Option<&Lua>) -> FinderFn<V>;
}
