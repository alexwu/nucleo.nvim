use std::{
    fs::File,
    ops::DerefMut,
    path::Path,
    sync::{mpsc, Arc},
};

use ignore::{types::TypesBuilder, DirEntry, WalkBuilder, WalkState};
use log::LevelFilter;
use mlua::prelude::*;
use nucleo::Injector;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use picker::Picker;
use simplelog::{Config, WriteLogger};
use std::env::current_dir;
use tokio::runtime::Runtime;

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

pub async fn fuzzy_file(lua: &Lua, params: (String, String)) -> LuaResult<LuaValue> {
    log::info!("fuzzy_file: {}, {}", params.0, params.1);
    // if params.1 != file_finder::finder().cwd {
    //     file_finder::update_cwd(&params.1);
    //     file_finder::parallel_files(&params.1, true);
    // }
    log::info!("fuzzy_file: {}, {}, after if statement", params.0, params.1);

    file_finder::matches(&params.0).into_lua(lua)
}

pub async fn fuzzy_file_callback(lua: &Lua, params: (String, String)) -> LuaResult<LuaValue> {
    log::info!("fuzzy_file: {}, {}", params.0, params.1);
    if params.1 != file_finder::finder().cwd {
        file_finder::update_cwd(&params.1);
        file_finder::parallel_files(&params.1, true);
    }
    log::info!("fuzzy_file: {}, {}, after if statement", params.0, params.1);

    file_finder::matches(&params.0).into_lua(lua)
}

fn nvim_api(lua: &Lua) -> LuaResult<LuaTable> {
    lua.globals().get::<&str, LuaTable>("vim")?.get("api")
}

pub fn nvim_buf_set_lines(lua: &Lua, params: (i64, i64, i64, bool, Vec<String>)) -> LuaResult<()> {
    nvim_api(lua)?
        .get::<&str, LuaFunction>("nvim_buf_set_lines")?
        .call::<_, ()>(params)
        .map_err(|err| err.into())
}

// pub fn init_picker(lua: &Lua, params: (String,)) -> LuaResult<Arc<Mutex<Picker>>> {
//     log::info!("Beginning to initialize picker");
//     // let mut picker = Picker::new(params.0.clone());
//     // let runtime = Runtime::new()?;
//     // runtime.spawn(async move {
//     // picker.populate_picker(&params.0, true);
//     // });
//     log::info!("About to return picker");
//
//     Ok(Arc::new(Mutex::new(Picker::default())))
// }
// pub fn init_picker(lua: &Lua, params: (String,)) -> LuaResult<LazyMutex<Picker>> {
//     Ok(PICKER)
// }

pub fn populate_injector(injector: Injector<String>, cwd: String, git_ignore: bool) {
    log::info!("Populating picker with {}", &cwd);
    let runtime = Runtime::new().expect("Failed to create runtime");

    let (tx, rx) = mpsc::channel::<String>();
    let add_to_injector_thread = std::thread::spawn(move || -> anyhow::Result<()> {
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
    // exports.set("fuzzy_match", lua.create_function(fuzzy)?)?;
    // exports.set("fuzzy_match", lua.create_function(fuzzy_match)?)?;
    // exports.set(
    //     "fuzzy_match_with_scores",
    //     lua.create_function(fuzzy_with_scores)?,
    // )?;
    // exports.set("files", lua.create_function(files)?)?;
    // exports.set("set_picker_items", lua.create_function(set_picker_items)?)?;
    // exports.set("matches", lua.create_function(matches)?)?;
    // exports.set("update_query", lua.create_function(update_query)?)?;
    // exports.set("restart_picker", lua.create_function(restart_picker)?)?;
    //
    // exports.set(
    //     "init_file_finder",
    //     lua.create_async_function(init_file_finder)?,
    // )?;
    // exports.set("fuzzy_file", lua.create_async_function(fuzzy_file)?)?;
    // exports.set("init_picker", lua.create_function(|_, params| {})?)?;
    exports.set(
        "Picker",
        lua.create_function(move |_, ()| Ok(picker.clone()))?,
    )?;

    // exports.set("Picker", picker::Picker);

    Ok(exports)
}
