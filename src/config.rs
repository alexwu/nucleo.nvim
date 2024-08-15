use std::str::FromStr;

use log::LevelFilter;
use mlua::{prelude::*, FromLua};
use partially::Partial;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use strum::{Display, EnumIs, EnumString};

use crate::injector::FromPartial;

#[derive(Clone, Debug, Partial, Default, Serialize, Deserialize)]
#[partially(derive(Default, Debug, Clone, Serialize, Deserialize))]
pub struct Config {
    log_level: LogLevel,
    sort_direction: SortDirection,
    selection_strategy: SelectionStrategy,
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    pub fn selection_strategy(&self) -> SelectionStrategy {
        self.selection_strategy
    }
}

impl FromLua for PartialConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;

        Ok(PartialConfig {
            log_level: table.get("log_level")?,
            sort_direction: table.get("sort_direction")?,
            selection_strategy: table.get("selection_strategy")?,
        })
    }
}

impl From<PartialConfig> for Config {
    fn from(value: PartialConfig) -> Self {
        Config::from_partial(value)
    }
}

/// These match vim.log.levels
#[derive(Clone, Debug, Default, Copy, EnumIs, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    #[default]
    Off = 5,
}

impl FromLua for LogLevel {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Off => LevelFilter::Off,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Default, PartialEq, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Default, PartialEq, EnumString, Display, EnumIs,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SelectionStrategy {
    #[default]
    Reset,
    Follow,
}

impl FromLua for SelectionStrategy {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            mlua::Value::String(str) => {
                let direction = match SelectionStrategy::from_str(&str.to_str()?) {
                    Ok(direction) => direction,
                    Err(_) => SelectionStrategy::default(),
                };
                Ok(direction)
            }
            _ => Ok(SelectionStrategy::default()),
        }
    }
}
impl FromLua for SortDirection {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for SortDirection {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue> {
        lua.to_value(&self)
    }
}
