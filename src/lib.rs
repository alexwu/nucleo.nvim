use std::{
    env::current_dir,
    fs::{self, File},
};

use directories::ProjectDirs;
use injector::FromPartial;
use mlua::prelude::*;
use simplelog::{Config, WriteLogger};
use sources::{
    diagnostics,
    files::{self, PartialFileConfig},
    git::{self, PartialStatusConfig},
};

use crate::util::align_str;

mod buffer;
mod config;
mod entry;
mod error;
mod injector;
mod matcher;
mod nucleo;
mod nvim;
mod pattern;
mod picker;
mod previewer;
mod sorted_vec;
mod sorter;
mod sources;
mod util;
mod window;

fn setup(opts: Option<config::PartialConfig>) -> anyhow::Result<()> {
    let config = config::Config::from_partial(opts.unwrap_or_default());

    let proj_dirs = ProjectDirs::from("", "bombeelu-labs", "nucleo")
        .expect("Unable to determine project directory");
    fs::create_dir_all(proj_dirs.cache_dir())?;
    let _ = WriteLogger::init(
        config.log_level().into(),
        Config::default(),
        File::create(proj_dirs.cache_dir().join("nucleo.log")).unwrap(),
    );
    log::info!(
        "Initialized logger with level {:?} at: {}",
        config.log_level(),
        current_dir().expect("Unable get current dir").display()
    );

    Ok(())
}

#[mlua::lua_module]
fn nucleo_rs(lua: &'static Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set(
        "setup",
        LuaFunction::wrap(|lua, params: (LuaValue,)| {
            let options: Option<config::PartialConfig> = lua.from_value(params.0)?;
            setup(options).into_lua_err()
        }),
    )?;

    exports.set(
        "FilePicker",
        LuaFunction::wrap(|_, params: (Option<PartialFileConfig>,)| {
            files::create_picker(params.0).into_lua_err()
        }),
    )?;
    exports.set(
        "CustomPicker",
        LuaFunction::wrap(|_, params: (sources::lua_tables::Source,)| {
            Ok(sources::lua_tables::Source::picker(
                params.0,
                Default::default(),
            ))
        }),
    )?;
    exports.set(
        "DiagnosticsPicker",
        LuaFunction::wrap(|_, params: (diagnostics::Source,)| {
            diagnostics::create_picker(params.0).into_lua_err()
        }),
    )?;
    exports.set(
        "GitStatusPicker",
        LuaFunction::wrap(|_, params: (Option<PartialStatusConfig>,)| {
            git::create_picker(params.0).into_lua_err()
        }),
    )?;
    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;
    exports.set(
        "align_str",
        LuaFunction::wrap(|lua, params: (String, LuaValue<'_>, u32, String, u32)| {
            let indices: Vec<(u32, u32)> = lua.from_value(params.1)?;

            let (display, adjusted_indices) =
                align_str(&params.0, &indices, params.2, &params.3, params.4);

            let table = lua.create_table()?;
            table.push(display)?;
            table.push(lua.to_value(&adjusted_indices)?)?;
            Ok(table)
        }),
    )?;

    Ok(exports)
}
