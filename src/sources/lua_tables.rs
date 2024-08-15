use std::{fmt::Debug, sync::Arc};

use mlua::{FromLua, Lua, LuaSerdeExt};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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

    fn build_injector(&self, _lua: Option<&Lua>) -> crate::injector::FinderFn<Data<Blob>> {
        let entries = self.results.clone();
        Arc::new(move |tx| {
            entries.par_iter().for_each(|entry| {
                let _ = tx.send(entry.clone());
            });
            Ok(())
        })
    }
}

impl FromLua for Source {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
