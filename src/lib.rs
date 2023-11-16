use std::borrow::Cow;
use std::env::current_dir;
use std::io::BufReader;
use std::{fs::File, sync::Arc};

use log::LevelFilter;
use mlua::prelude::*;
use parking_lot::Mutex;
use picker::{FileEntry, Picker};
use ropey::Rope;
use simplelog::{Config, WriteLogger};

mod injector;
mod picker;

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

pub fn preview_file(lua: &Lua, params: (Option<String>,usize)) -> LuaResult<String> {
    match params.0 {
        Some(path) => {
            log::info!("Previewing file {}", path);
            let  text = Rope::from_reader(BufReader::new(File::open(path)?))?;
            let end_line = text.len_lines().min(params.1);
            let start_idx = text.line_to_char(0);
            let end_idx = text.line_to_char(end_line);

            Ok(text.slice(start_idx..end_idx).to_string())

            // todo!()
        }
        None => Ok(String::new()),
    }
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
    exports.set("preview_file", lua.create_function(preview_file)?)?;

    Ok(exports)
}
