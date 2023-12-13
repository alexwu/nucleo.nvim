use std::{fs::File, sync::Arc};

use anyhow::bail;
use crossbeam_channel::Sender;
use entry::{CustomEntry, Entry};
use log::LevelFilter;
use mlua::prelude::*;
use picker::{Data, FileEntry, Picker};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::{Config, WriteLogger};
use sources::{
    files::{self, FileConfig, PartialFileConfig},
    lsp::Diagnostic,
};
use tokio::runtime::Runtime;

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

pub enum LuaFinder {
    Table(LuaTable<'static>),
    Function(LuaFunction<'static>),
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
pub fn init_lua_picker(
    lua: &'static Lua,
    params: (LuaValue<'static>,),
) -> LuaResult<Picker<Diagnostic>> {
    let rt = Runtime::new()?;
    let mut picker: Picker<Diagnostic> = Picker::new(picker::Config::default());
    let local = tokio::task::LocalSet::new();
    let results = match params.0.clone() {
        LuaValue::LightUserData(_) => todo!(),
        LuaValue::Table(_) => todo!(),
        LuaValue::Function(finder) => {
            // rt.block_on(async {
            //     let finder = finder.clone();
            picker.populate_with_local(move |tx| {
                //         local.run_until(async {
                let results = finder.call::<_, Vec<Diagnostic>>(());
                log::info!("please {:?}", results);
                match results {
                    Ok(entries) => entries.par_iter().for_each(|entry| {
                        let _ = tx.send(Diagnostic::from_diagnostic(entry.clone()));
                    }),
                    Err(_) => todo!(),
                }
                //         }).await;
            });
            // });
        }
        LuaValue::Thread(_) => todo!(),
        _ => todo!("Invalid finder"),
    };

    Ok(picker)
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
    let mut picker: Picker<CustomEntry> = Picker::new(picker::Config::default());

    // picker.populate(results);

    Ok(picker)
}

// pub fn init_file_picker(lua: &Lua, params: ()) -> LuaResult<Picker<files::Value>> {
pub fn init_file_picker(
    lua: &Lua,
    params: (Option<PartialFileConfig>,),
) -> LuaResult<Picker<files::Value>> {
    files::create_picker(params.0).into_lua_err()
}

#[mlua::lua_module]
fn nucleo_rs(lua: &'static Lua) -> LuaResult<LuaTable> {
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
    exports.set("LuaPicker", lua.create_function(init_lua_picker)?)?;
    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;

    Ok(exports)
}
