use std::{fmt::Debug, str::FromStr};

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Deserializer, Serialize};
use strum::{Display, EnumIs, EnumString};

use crate::{entry::IntoUtf32String, injector::FinderFn};

pub mod custom;
pub mod diagnostics;
pub mod files;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "git")]
pub mod git_hunks;
mod lua_function;
pub mod lua_tables;
pub mod source;

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
    #[cfg(feature = "git")]
    #[strum(serialize = "builtin.git_status")]
    GitStatus,
    #[cfg(feature = "git")]
    #[strum(serialize = "builtin.git_hunks")]
    GitHunks,
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

impl FromLua for Sources {
    fn from_lua(value: mlua::Value, lua: &'_ Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for Sources {
    fn into_lua(self, lua: &'_ Lua) -> mlua::Result<mlua::Value> {
        self.to_string().into_lua(lua)
    }
}

pub trait Populator<T, U, V>
where
    T: Debug + Serialize + for<'a> Deserialize<'a>,
    U: Debug + Default + Serialize + for<'a> Deserialize<'a>,
    V: IntoUtf32String,
{
    fn name(&self) -> Sources;
    fn kind(&self) -> SourceKind;
    fn update_config(&mut self, config: U);

    fn build_injector(&mut self, lua: Option<&Lua>) -> FinderFn<V>;
}
