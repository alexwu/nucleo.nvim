use std::fs::File;

use log::LevelFilter;
use mlua::prelude::*;
use simplelog::{Config, WriteLogger};

mod file_finder;
mod fuzzy;
mod picker;

pub fn files(lua: &Lua, params: String) -> LuaResult<LuaValue> {
    fuzzy::files(&params, true).into_lua(lua)
}

pub fn fuzzy(lua: &Lua, params: (String, Vec<String>)) -> LuaResult<LuaValue> {
    let mut result = fuzzy::fuzzy_match(&params.0, params.1, true);
    result.sort_by(|a, b| b.1.cmp(&a.1));

    let table = Vec::from_iter(result.into_iter().map(|(item, _)| item));

    table.into_lua(lua)
}

pub fn fuzzy_with_scores(lua: &Lua, params: (String, Vec<String>)) -> LuaResult<LuaValue> {
    let mut result = fuzzy::fuzzy_match(&params.0, params.1, true);
    result.sort_by(|a, b| b.1.cmp(&a.1));

    let table = lua.create_table()?;
    result.into_iter().for_each(|(path, score)| {
        // item.into_lua(lua).ok()
        table.set(path, score).ok();
    });

    table.into_lua(lua)
}

pub fn set_picker_items(_lua: &Lua, params: Vec<String>) -> LuaResult<()> {
    fuzzy::set_picker_items(params);
    Ok(())
}

pub fn update_query(_lua: &Lua, params: String) -> LuaResult<()> {
    fuzzy::update_query(&params);
    Ok(())
}

pub fn matches(lua: &Lua, _params: ()) -> LuaResult<LuaValue> {
    fuzzy::matches().into_lua(lua)
}

pub fn restart_picker(_lua: &Lua, _params: ()) -> LuaResult<()> {
    fuzzy::restart_picker();
    Ok(())
}

pub fn fuzzy_match(lua: &Lua, params: (String,)) -> LuaResult<LuaValue> {
    fuzzy::update_query(&params.0);

    fuzzy::matches().into_lua(lua)
}

pub async fn init_file_finder(lua: &Lua, params: (String,)) -> LuaResult<()> {
    log::info!("init_file_finder");

    if params.0 != file_finder::finder().cwd {
        file_finder::update_cwd(&params.0);
        file_finder::parallel_files(&params.0, true).await;
    }
    log::info!("init_file_finder: after if statement");

    Ok(())
}

pub async fn fuzzy_file(lua: &Lua, params: (String, String)) -> LuaResult<LuaValue> {
    log::info!("fuzzy_file: {}, {}", params.0, params.1);
    if params.1 != file_finder::finder().cwd {
        file_finder::update_cwd(&params.1);
        file_finder::parallel_files(&params.1, true).await;
    }
    log::info!("fuzzy_file: {}, {}, after if statement", params.0, params.1);

    file_finder::matches(&params.0).into_lua(lua)
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
    // exports.set("fuzzy_match", lua.create_function(fuzzy)?)?;
    exports.set("fuzzy_match", lua.create_function(fuzzy_match)?)?;
    exports.set(
        "fuzzy_match_with_scores",
        lua.create_function(fuzzy_with_scores)?,
    )?;
    exports.set("files", lua.create_function(files)?)?;
    exports.set("set_picker_items", lua.create_function(set_picker_items)?)?;
    exports.set("matches", lua.create_function(matches)?)?;
    exports.set("update_query", lua.create_function(update_query)?)?;
    exports.set("restart_picker", lua.create_function(restart_picker)?)?;

    exports.set(
        "init_file_finder",
        lua.create_async_function(init_file_finder)?,
    )?;
    exports.set("fuzzy_file", lua.create_async_function(fuzzy_file)?)?;

    Ok(exports)
}
