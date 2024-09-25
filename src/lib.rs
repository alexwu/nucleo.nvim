use std::{
    env::current_dir,
    fs::{self, File},
};

use directories::ProjectDirs;
use injector::FromPartial;
use mlua::prelude::*;
use simplelog::{Config, WriteLogger};
use sources::lua_value;

use crate::error::Result;
#[cfg(feature = "git")]
use crate::sources::git::{self, PartialStatusConfig};
#[cfg(feature = "git")]
use crate::sources::git_hunks;
use crate::sources::{diagnostics, files, Sources};
use crate::util::align_str;

mod buffer;
mod config;
mod entry;
mod error;
mod injector;
mod lua;
mod matcher;
mod nucleo;
mod pattern;
mod picker;
mod previewer;
mod sorted_vec;
mod sorter;
mod sources;
mod util;
mod window;

fn setup(opts: Option<config::PartialConfig>) -> Result<()> {
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
fn nucleo_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set(
        "setup",
        LuaFunction::wrap(|lua, params: (LuaValue,)| {
            let options: Option<config::PartialConfig> = lua.from_value(params.0)?;
            setup(options).into_lua_err()
        }),
    )?;

    exports.set(
        "Picker",
        LuaFunction::wrap(|lua, params: (LuaValue,)| {
            let table = LuaTable::from_lua(params.0.clone(), lua)?;
            let name: Sources = table.get("name")?;
            let config: Option<LuaValue> = table.get("config")?;

            match name {
                Sources::Files => {
                    let opts: Option<files::PartialFileConfig> =
                        config.and_then(|c| files::PartialFileConfig::from_lua(c, lua).ok());

                    files::create_picker(opts).into_lua_err()?.into_lua(lua)
                }
                #[cfg(feature = "git")]
                Sources::GitStatus => {
                    let opts: Option<PartialStatusConfig> =
                        config.and_then(|c| lua.from_value(c).ok()?);

                    git::create_picker(opts).into_lua_err()?.into_lua(lua)
                }
                #[cfg(feature = "git")]
                Sources::GitHunks => {
                    let opts: Option<git_hunks::PartialHunkConfig> =
                        config.and_then(|c| lua.from_value(c).ok()?);

                    git_hunks::Source::picker(opts)
                        .into_lua_err()?
                        .into_lua(lua)
                }
                Sources::Diagnostics => {
                    let source = diagnostics::Source::from_lua(params.0, lua)?;

                    diagnostics::create_picker(source)
                        .into_lua_err()?
                        .into_lua(lua)
                }
                Sources::Custom(_) => {
                    let source: lua_value::Source = lua_value::Source::from_lua(params.0, lua)?;

                    let picker = lua_value::create_picker(source);

                    picker.into_lua(lua)
                }
            }
        }),
    )?;

    exports.set(
        "Previewer",
        LuaFunction::wrap(|_, ()| Ok(previewer::Previewer::new())),
    )?;
    exports.set(
        "align_str",
        LuaFunction::wrap(|lua, params: (String, LuaValue, u32, String, u32)| {
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
