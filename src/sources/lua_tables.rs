use std::fmt::Debug;

use mlua::{FromLua, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use super::{Populator, Sources};
use crate::{
    entry::Data,
    picker::{Blob, Picker},
};

#[derive(Debug, Clone, Serialize, Deserialize, buildstructor::Builder)]
pub struct Source {
    name: String,
    config: Blob,
    results: Vec<Data<Blob>>,
}

impl Source {
    pub fn picker(source: Self, config: crate::config::Config) -> Picker<Blob, Blob, Source> {
        Picker::builder().source(source).config(config).build()
    }
}

impl Populator<Blob, Blob, Data<Blob>> for Source {
    fn name(&self) -> Sources {
        Sources::Custom(self.name.clone())
    }

    fn kind(&self) -> super::SourceKind {
        super::SourceKind::Lua
    }

    fn update_config(&mut self, config: Blob) {
        self.config = config;
    }

    fn build_injector(&mut self, _lua: Option<&Lua>) -> crate::injector::FinderFn<Data<Blob>> {
        todo!("build_injector for lua_tables")
        // let entries = std::mem::take(&mut self.results);
        // Arc::new(move |tx| {
        //     entries.into_iter().for_each(|entry| {
        //         let _ = tx.send(entry);
        //     });
        //     Ok(())
        // })
    }
}

impl FromLua for Source {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
