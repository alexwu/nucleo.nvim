use std::sync::Arc;

use buildstructor::Builder;
use mlua::prelude::*;
use mlua::{FromLua, Function, Lua, LuaSerdeExt};
use partially::Partial;
use serde::{Deserialize, Serialize};

use super::{Populator, SourceKind, Sources};
use crate::entry::{Data, DataKind};
use crate::picker::Picker;
use crate::{config, injector::FromPartial};

#[derive(Debug, Clone, Serialize, Deserialize, Default, Partial)]
#[partially(derive(Clone, Debug, Serialize, Deserialize, Default))]
pub struct Config {
    #[serde(flatten, default)]
    picker_config: config::PartialConfig,
}

impl IntoLua for Config {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        lua.to_value(&self)
    }
}

impl FromLua for PartialConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl FromLua for Config {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Value {
    #[serde(flatten, default)]
    inner: LuaValue,
}

impl IntoLua for Value {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.inner.into_lua(lua)
    }
}

impl FromLua for Value {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        Ok(Value { inner: value })
    }
}

impl From<LuaValue> for Value {
    fn from(value: LuaValue) -> Self {
        Value { inner: value }
    }
}

#[derive(Debug, Clone)]
pub enum Finder {
    Function(Function),
    Table(mlua::Table),
}

impl IntoLua for Finder {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Finder::Function(val) => val.into_lua(lua),
            Finder::Table(val) => val.into_lua(lua),
        }
    }
}

#[derive(Debug, Clone, Builder, Serialize)]
pub struct Source {
    name: String,
    config: Config,
    #[serde(skip_serializing)]
    // finder: Arc<Finder>,
    results: Vec<Data<i32>>,
}

// impl FromLua for Source {
//     fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
//         let table = value
//             .as_table()
//             .ok_or_else(|| anyhow::anyhow!("Source wasn't given a table!"))
//             .into_lua_err()?;
//
//         let finder = match table.get::<LuaValue>("finder")? {
//             LuaValue::Function(thunk) => Finder::Function(thunk),
//             LuaValue::Table(table) => Finder::Table(table),
//             _ => todo!("Failed to implement finder"),
//         };
//
//         let name: String = match table.get::<LuaValue>("name")? {
//             LuaValue::String(val) => val.to_string_lossy(),
//             _ => todo!("NAME"),
//         };
//
//         let partial_config: PartialConfig = lua.from_value(table.get::<LuaValue>("config")?)?;
//         log::debug!("lua_value partial config: {:?}", &partial_config);
//         let config = Config::from_partial(partial_config);
//
//         Ok(Source::builder()
//             .name(name)
//             .config(config)
//             .finder(Arc::new(finder))
//             .build())
//     }
// }

impl FromLua for Source {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        let name: String = match table.get::<LuaValue>("name")? {
            LuaValue::String(val) => val.to_string_lossy(),
            _ => todo!("NAME"),
        };

        let partial_config: PartialConfig = lua.from_value(table.get::<LuaValue>("config")?)?;
        log::debug!("lua_value partial config: {:?}", &partial_config);
        let config = Config::from_partial(partial_config);
        let results: Vec<Data<i32>> = lua.from_value(table.get::<LuaValue>("results")?)?;

        Ok(Source::builder()
            .name(name)
            .config(config)
            .results(results)
            .build())
    }
}

// impl IntoLua for Source {
//     fn into_lua(self, _lua: &Lua) -> LuaResult<LuaValue> {
//         todo!()
//     }
// }

impl Populator<i32, Config, Data<i32>> for Source {
    fn name(&self) -> Sources {
        Sources::Custom(self.name.clone())
    }

    fn kind(&self) -> super::SourceKind {
        SourceKind::Lua
    }

    fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    fn build_injector(&mut self, _lua: Option<&Lua>) -> crate::injector::FinderFn<Data<i32>> {
        let entries = self.results.clone();
        Arc::new(move |tx| {
            entries.clone().into_iter().for_each(|entry| {
                let _ = tx.send(entry);
            });
            Ok(())
        })
        // let finder = self.results.clone();
        // let results: mlua::Result<Vec<Data<Value>>> = match finder.as_ref() {
        //     Finder::Function(thunk) => thunk.call(()),
        //     // Finder::Table(table) => Ok(table.clone()),
        //     Finder::Table(table) => todo!("UGH"),
        // };
        //
        // let entries: Vec<Data<Value>> = match results {
        //     Ok(entries) => entries,
        //     Err(error) => {
        //         log::error!("Errored calling finder fn: {}", error);
        //         vec![]
        //     }
        // };
        //
        // Arc::new(move |tx| {
        //     entries.clone().into_iter().for_each(|entry| {
        //         // let ordinal = match &entry {
        //         //     LuaValue::Table(val) => val
        //         //         .get::<&str, String>("ordinal")
        //         //         .expect("Failed getting ordinal"),
        //         //     val => val.to_string().expect("Failed ordinalizing"),
        //         // };
        //         // let table = match &entry {
        //         //     LuaValue::Table(val) => val,
        //         //     val => val.to_string().expect("Failed ordinalizing"),
        //         // };
        //
        //         // let v = match Data::from_lua(entry, lua.expect("Lua not here")) {
        //         //     Ok(val) => val,
        //         //     Err(_) => todo!("ER"),
        //         // };
        //
        //         // let value = table.get::<&str, LuaValue>("value")?;
        //
        //         // let data: Data<Value> = Data::builder()
        //         //     .kind(DataKind::Custom("TODO".into()))
        //         //     .ordinal(ordinal)
        //         //     .value(entry.value)
        //         //     .build();
        //
        //         let _ = tx.send(entry);
        //         // Ok(())
        //     });
        //     Ok(())
        // })
    }
}

pub fn create_picker(source: Source) -> impl IntoLua {
    let picker_config = source.config.picker_config.clone();
    let picker: Picker<i32, Config, Source> = Picker::builder()
        .multi_sort(false)
        .source(source)
        .config(picker_config)
        .build();

    picker
}
