use mlua::{FromLua, Function, Lua, LuaSerdeExt, RegistryKey, Value};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};

use crate::{
    entry::Data,
    picker::{self, Blob, Picker},
};

use super::Populator;

#[derive(FromLua, Debug, Clone, Serialize, Deserialize, buildstructor::Builder)]
pub struct Source {
    name: String,
    config: Blob,
    #[serde(skip)]
    function_key: Option<Arc<RegistryKey>>,
}

impl Source {
    pub fn picker(source: Self, config: picker::Config) -> Picker<Blob, Blob, Source> {
        Picker::builder().source(source).config(config).build()
    }
}

impl Populator<Blob, Blob, Data<Blob>> for Source {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn kind(&self) -> super::SourceKind {
        super::SourceKind::Lua
    }

    fn update_config(&mut self, config: Blob) {
        self.config = config;
    }

    fn build_injector(&self, lua: Option<&Lua>) -> crate::injector::FinderFn<Data<Blob>> {
        let key = self.function_key.clone().expect("No registry key stored!");
        let finder = lua
            .expect("No Lua object given!")
            .registry_value::<Function>(&key)
            .expect("Remember to make it so these return results!");
        let results = finder.call::<_, Value>(());
        let entries = match results {
            Ok(entries) => lua
                .expect("No lua!")
                .from_value::<Vec<Data<Blob>>>(entries)
                .expect("Error with diagnostics"),
            Err(error) => {
                log::error!("Errored calling finder fn: {}", error);
                Vec::new()
            }
        };

        Arc::new(move |tx| {
            entries.clone().into_iter().for_each(|entry| {
                let _ = tx.send(entry);
            });
            Ok(())
        })
    }
}
