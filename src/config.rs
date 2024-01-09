use std::str::FromStr;

use log::LevelFilter;
use mlua::{prelude::*, FromLua};
use partially::Partial;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use strum::{Display, EnumIs, EnumString};

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
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Default, PartialEq, EnumString, Display, EnumIs,
)]
#[strum(serialize_all = "snake_case")]
pub enum SelectionStrategy {
    #[default]
    Reset,
    Follow,
}

impl FromLua<'_> for SelectionStrategy {
    fn from_lua(value: LuaValue<'_>, _lua: &'_ Lua) -> LuaResult<Self> {
        match value {
            mlua::Value::String(str) => {
                let direction = match SelectionStrategy::from_str(str.to_str()?) {
                    Ok(direction) => direction,
                    Err(_) => SelectionStrategy::default(),
                };
                Ok(direction)
            }
            _ => Ok(SelectionStrategy::default()),
        }
    }
}
impl FromLua<'_> for SortDirection {
    fn from_lua(value: LuaValue<'_>, _lua: &'_ Lua) -> LuaResult<Self> {
        match value {
            mlua::Value::String(str) => {
                let direction = match SortDirection::from_str(str.to_str()?) {
                    Ok(direction) => direction,
                    Err(_) => SortDirection::Descending,
                };
                Ok(direction)
            }
            _ => Ok(SortDirection::Descending),
        }
    }
}

impl IntoLua<'_> for SortDirection {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue<'_>> {
        self.to_string().into_lua(lua)
    }
}

#[derive(Clone, Debug, Partial, Default, Serialize, Deserialize, FromLua)]
#[partially(derive(Default, Debug, Clone, Serialize, Deserialize, FromLua))]
pub struct Config {
    log_level: LogLevel,
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }
}
