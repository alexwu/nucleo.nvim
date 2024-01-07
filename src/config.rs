use log::LevelFilter;
use mlua::FromLua;
use partially::Partial;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use strum::EnumIs;

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
