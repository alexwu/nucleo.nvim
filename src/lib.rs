use std::{fs::File, sync::Arc};

use entry::{CustomEntry, Entry};
use log::LevelFilter;
use mlua::prelude::*;
use picker::{Data, FileEntry, Picker};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::{Config, WriteLogger};
use sources::files::{self, FileConfig};

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

    let picker = Picker::new::<FileEntry>(config.into());

    Ok(picker)
}

pub enum SourceKind {
    Builtin,
    Lua(SourceConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    name: String,
    results: Vec<Data<CustomEntry>>,
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
    // let results = params
    //     .0
    //     .results
    //     .into_par_iter()
    //     .map(|entry| Data {
    //         display: entry.display(),
    //         value: entry,
    //         selected: false,
    //         indices: vec![],
    //     })
    //     .collect();
    let mut picker: Picker<CustomEntry> = Picker::new::<CustomEntry>(picker::Config::default());

    // picker.populate(results);

    Ok(picker)
}

pub fn init_file_picker(lua: &Lua, params: ()) -> LuaResult<Picker<files::Value>> {
    // pub fn init_file_picker(lua: &Lua, params: (FileConfig,)) -> LuaResult<Picker<files::Value>> {
    let populator = files::injector(FileConfig::default());
    let mut picker: Picker<files::Value> = Picker::new::<files::Value>(picker::Config::default());

    picker.set_populator(Arc::new(move |tx| {
        populator(tx);
    }));
    // picker.populate_with(populator);

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
    exports.set("FilePicker", lua.create_function(init_file_picker)?)?;
    exports.set("CustomPicker", lua.create_function(init_custom_picker)?)?;
    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;

    Ok(exports)
}
