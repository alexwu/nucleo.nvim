use std::fs::File;

use entry::{CustomEntry, Entry};
use log::LevelFilter;
use mlua::prelude::*;
use picker::{FileEntry, Picker};
use serde::{Deserialize, Serialize};
use simplelog::{Config, WriteLogger};

mod buffer;
mod entry;
mod injector;
mod matcher;
mod picker;
mod previewer;
mod sources;

pub fn init_picker(
    _: &Lua,
    params: (Option<picker::PartialConfig>,),
) -> LuaResult<Picker<FileEntry>> {
    let config = match params.0 {
        Some(config) => config,
        None => picker::PartialConfig::default(),
    };

    let picker = Picker::new(config.into());

    Ok(picker)
}

pub enum SourceKind {
    Builtin,
    Lua(SourceConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    name: String,
    results: Vec<CustomEntry>,
}

impl FromLua<'_> for SourceConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;

        Ok(Self {
            name: table.get("name")?,
            results: table.get("results")?,
        })
    }
}

pub fn init_custom_picker(lua: &Lua, params: (SourceConfig,)) -> LuaResult<Picker<CustomEntry>> {
    // let results: Result<Vec<CustomEntry>, LuaError> = match table.get::<&str, LuaValue>("results") {
    //     Ok(val) => match val {
    //         // LuaValue::Function(func) => func.call::<_, Vec<CustomEntry>>(()),
    //         LuaValue::Function(func) => todo!(),
    //         // LuaValue::Table(_) => Ok(lua.from_value(val)),
    //         _ => Err("Invalid parameter type inside").into_lua_err(),
    //     },
    //     _ => Err("Invalid parameter type").into_lua_err(),
    // };
    let picker: Picker<CustomEntry> = Picker::new(picker::Config::default());

    Ok(picker)
}

#[mlua::lua_module]
fn nucleo_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("nucleo.log").unwrap(),
    );
    log::info!("Initialized logger");

    let exports = lua.create_table()?;

    exports.set("Picker", lua.create_function(init_picker)?)?;
    exports.set("CustomPicker", lua.create_function(init_custom_picker)?)?;
    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;

    Ok(exports)
}
