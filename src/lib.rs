use std::env::current_dir;
use std::{fs::File, sync::Arc};

use log::LevelFilter;
use mlua::prelude::*;
use parking_lot::Mutex;
use picker::{FileEntry, Picker};
use simplelog::{Config, WriteLogger};

mod injector;
mod picker;

fn nvim_api(lua: &Lua) -> LuaResult<LuaTable> {
    lua.globals().get::<&str, LuaTable>("vim")?.get("api")
}

pub fn nvim_buf_set_lines(lua: &Lua, params: (i64, i64, i64, bool, Vec<String>)) -> LuaResult<()> {
    nvim_api(lua)?
        .get::<&str, LuaFunction>("nvim_buf_set_lines")?
        .call::<_, ()>(params)
}

pub fn init_picker(
    _lua: &Lua,
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

    Ok(exports)
}
