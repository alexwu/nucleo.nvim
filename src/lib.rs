use std::env::current_dir;
use std::{fs::File, sync::Arc};

use log::LevelFilter;
use mlua::prelude::*;
use parking_lot::Mutex;
use picker::{FileEntry, Picker};
use simplelog::{Config, WriteLogger};

mod buffer;
mod injector;
mod picker;
mod previewer;

pub fn init_picker(
    _: &Lua,
    params: (Option<picker::Config>,),
) -> LuaResult<Arc<Mutex<Picker<FileEntry>>>> {
    let config = match params.0 {
        Some(config) => config,
        None => picker::Config::default(),
    };

    let cwd = match config.cwd {
        Some(cwd) => cwd,
        None => current_dir().unwrap().to_string_lossy().to_string(),
    };
    let picker = Arc::new(Mutex::new(Picker::new(cwd)));

    picker.lock().populate_files();

    Ok(picker)
}

#[mlua::lua_module]
fn nucleo_nvim(lua: &Lua) -> LuaResult<LuaTable> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("nucleo.log").unwrap(),
    );
    log::info!("Initialized logger");

    let exports = lua.create_table()?;

    exports.set("Picker", lua.create_function(init_picker)?)?;
    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;

    Ok(exports)
}
