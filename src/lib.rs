use std::{
    env::current_dir,
    fs::{self, File},
};

use directories::ProjectDirs;
use log::LevelFilter;
use mlua::prelude::*;
use simplelog::{Config, WriteLogger};
use sources::{
    diagnostics,
    files::{self, PartialFileConfig},
    git::{self, PartialStatusConfig},
};

use crate::util::align_str;

mod buffer;
mod entry;
mod error;
mod injector;
mod matcher;
mod nvim;
mod pattern;
mod picker;
mod previewer;
mod sorter;
mod sources;
mod util;
mod window;
mod nucleo;

// fn init_lua_picker(
//     lua: &'static Lua,
//     params: (LuaValue<'static>,),
// ) -> LuaResult<Picker<Diagnostic, diagnostics::Config, diagnostics::Source>> {
//     let mut picker = diagnostics::create_picker().into_lua_err()?;
//     match params.0.clone() {
//         LuaValue::LightUserData(_) => todo!(),
//         LuaValue::Table(_) => todo!("Table not yet implemented"),
//         LuaValue::Function(finder) => picker
//             .populate_with_local(lua, move |tx| {
//                 let results = finder.call::<_, LuaValue>(());
//                 match results {
//                     Ok(entries) => {
//                         let entries = lua
//                             .from_value::<Vec<Diagnostic>>(entries)
//                             .expect("Error with diagnostics");
//                         log::info!("{:?}", entries);
//                         rayon::spawn(move || {
//                             entries.into_iter().for_each(|entry| {
//                                 let _ = tx.send(entry.into());
//                             });
//                         });
//
//                         Ok(())
//                     }
//                     Err(error) => {
//                         log::error!("Errored calling finder fn: {}", error);
//                         bail!(error)
//                     }
//                 }
//             })
//             .into_lua_err(),
//         LuaValue::Thread(_) => todo!(),
//         _ => todo!("Invalid finder"),
//     }?;
//
//     Ok(picker)
// }

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
