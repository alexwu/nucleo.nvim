use std::{
    env::current_dir,
    fs::{self, File},
};

use directories::ProjectDirs;
use entry::CustomEntry;
use log::LevelFilter;
use mlua::prelude::*;
use picker::{Blob, Data, FileEntry, Picker};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use simplelog::{Config, WriteLogger};
use sources::files::{self, PartialFileConfig, PreviewOptions};

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
) -> LuaResult<Picker<FileEntry, PreviewOptions>> {
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
    results: Vec<Data<CustomEntry, Blob>>,
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

pub fn call_or_get<T>(lua: &Lua, val: LuaValue, field: &str) -> LuaResult<T>
where
    T: for<'a> IntoLua<'a> + for<'a> FromLua<'a> + for<'a> Deserialize<'a>,
{
    let table = LuaTable::from_lua(val, lua)?;
    match table.get(field)? {
        LuaValue::Function(func) => func.call::<_, T>(()),
        val => lua.from_value(val),
    }
}

pub fn init_lua_picker(
    _lua: &'static Lua,
    params: (LuaValue<'static>,),
) -> LuaResult<Picker<CustomEntry, Blob>> {
    let mut picker: Picker<CustomEntry, Blob> = Picker::new(picker::Config::default());
    match params.0.clone() {
        LuaValue::LightUserData(_) => todo!(),
        LuaValue::Table(_) => todo!(),
        LuaValue::Function(finder) => {
            picker.populate_with_local(move |tx| {
                let results = finder.call::<_, Vec<CustomEntry>>(());
                log::info!("please {:?}", results);
                match results {
                    Ok(entries) => entries.par_iter().for_each(|entry| {
                        let _ = tx.send(entry.clone().into());
                    }),
                    Err(_) => todo!(),
                }
            });
        }
        LuaValue::Thread(_) => todo!(),
        _ => todo!("Invalid finder"),
    };

    Ok(picker)
}

pub fn init_custom_picker(
    _lua: &Lua,
    _params: (SourceConfig,),
) -> LuaResult<Picker<CustomEntry, Blob>> {
    let picker: Picker<CustomEntry, Blob> = Picker::new(picker::Config::default());

    // picker.populate(results);

    Ok(picker)
}

pub fn init_file_picker(
    _lua: &Lua,
    params: (Option<PartialFileConfig>,),
) -> LuaResult<Picker<files::Value, PreviewOptions>> {
    files::create_picker(params.0).into_lua_err()
}

#[mlua::lua_module]
fn nucleo_rs(lua: &'static Lua) -> LuaResult<LuaTable> {
    let proj_dirs = ProjectDirs::from("", "bombeelu-labs", "nucleo")
        .expect("Unable to determine project directory");
    fs::create_dir_all(proj_dirs.cache_dir())?;
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(proj_dirs.cache_dir().join("nucleo.log")).unwrap(),
    );
    log::info!(
        "Initialized logger at: {}",
        current_dir().expect("Unable get current dir").display()
    );

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
