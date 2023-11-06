use std::env::current_dir;
use std::{
    fs::File,
    path::Path,
    sync::{mpsc, Arc},
};

use ignore::{types::TypesBuilder, DirEntry, WalkBuilder, WalkState};
use log::LevelFilter;
use mlua::prelude::*;
use nucleo::Injector;
use parking_lot::Mutex;
use picker::Picker;
use simplelog::{Config, WriteLogger};
use tokio::runtime::Runtime;

// mod file_finder;
mod picker;

// pub async fn fuzzy_file(lua: &Lua, params: (String, String)) -> LuaResult<LuaValue> {
//     log::info!("fuzzy_file: {}, {}", params.0, params.1);
//     // if params.1 != file_finder::finder().cwd {
//     //     file_finder::update_cwd(&params.1);
//     //     file_finder::parallel_files(&params.1, true);
//     // }
//     log::info!("fuzzy_file: {}, {}, after if statement", params.0, params.1);
//
//     file_finder::matches(&params.0).into_lua(lua)
// }
//
// pub async fn fuzzy_file_callback(lua: &Lua, params: (String, String)) -> LuaResult<LuaValue> {
//     log::info!("fuzzy_file: {}, {}", params.0, params.1);
//     if params.1 != file_finder::finder().cwd {
//         file_finder::update_cwd(&params.1);
//         file_finder::parallel_files(&params.1, true);
//     }
//     log::info!("fuzzy_file: {}, {}, after if statement", params.0, params.1);
//
//     file_finder::matches(&params.0).into_lua(lua)
// }
//
fn nvim_api(lua: &Lua) -> LuaResult<LuaTable> {
    lua.globals().get::<&str, LuaTable>("vim")?.get("api")
}

pub fn nvim_buf_set_lines(lua: &Lua, params: (i64, i64, i64, bool, Vec<String>)) -> LuaResult<()> {
    nvim_api(lua)?
        .get::<&str, LuaFunction>("nvim_buf_set_lines")?
        .call::<_, ()>(params)
}

pub fn populate_injector(injector: Injector<String>, cwd: String, git_ignore: bool) {
    log::info!("Populating picker with {}", &cwd);
    let runtime = Runtime::new().expect("Failed to create runtime");

    let (tx, rx) = mpsc::channel::<String>();
    let _add_to_injector_thread = std::thread::spawn(move || -> anyhow::Result<()> {
        for val in rx.iter() {
            injector.push(val.clone(), |dst| dst[0] = val.into());
        }
        Ok(())
    });

    let _ = runtime.spawn(async move {
        let dir = Path::new(&cwd);
        log::info!("Spawning file searcher...");
        let mut walk_builder = WalkBuilder::new(dir.clone());
        walk_builder
            .hidden(true)
            .follow_links(true)
            .git_ignore(git_ignore)
            .sort_by_file_name(|name1, name2| name1.cmp(name2));
        let mut type_builder = TypesBuilder::new();
        type_builder
            .add(
                "compressed",
                "*.{zip,gz,bz2,zst,lzo,sz,tgz,tbz2,lz,lz4,lzma,lzo,z,Z,xz,7z,rar,cab}",
            )
            .expect("Invalid type definition");
        type_builder.negate("all");
        let excluded_types = type_builder
            .build()
            .expect("failed to build excluded_types");
        walk_builder.types(excluded_types);
        walk_builder.build_parallel().run(|| {
            let cwd = cwd.clone();
            let tx = tx.clone();
            Box::new(move |path: Result<DirEntry, ignore::Error>| -> WalkState {
                match path {
                    Ok(file) if file.path().is_file() => {
                        let val = file
                            .path()
                            .strip_prefix(&cwd)
                            .expect("Failed to strip prefix")
                            .to_str()
                            .expect("Failed to convert path to string")
                            .to_string();
                        log::info!("Adding {}", &val);
                        // injector.push(val.clone(), |dst| dst[0] = val.into());
                        match tx.send(val.clone()) {
                            Ok(_) => WalkState::Continue,
                            Err(_) => WalkState::Skip,
                        }
                    }
                    Ok(_) => WalkState::Continue,
                    Err(_) => WalkState::Skip,
                }
            })
        });
    });

    log::info!("After spawning file searcher...");
}
#[mlua::lua_module]
fn nucleo_nvim(lua: &Lua) -> LuaResult<LuaTable> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("nucleo.log").unwrap(),
    );
    log::info!("Initialized logger");

    let dir = current_dir().unwrap();
    let picker = Arc::new(Mutex::new(Picker::new(dir.to_string_lossy().to_string())));

    let injector = picker.lock().matcher.injector();
    std::thread::spawn(move || {
        populate_injector(injector, dir.to_string_lossy().to_string(), true);
    });

    let exports = lua.create_table()?;

    exports.set(
        "Picker",
        lua.create_function(move |_, ()| Ok(picker.clone()))?,
    )?;

    Ok(exports)
}
