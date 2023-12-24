use mlua::FromLua;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};

use crate::picker::{self, Blob, Data, Picker};

use super::Populator;

#[derive(FromLua, Debug, Clone, Serialize, Deserialize, buildstructor::Builder)]
pub struct Source {
    name: String,
    config: Blob,
    results: Vec<Data<Blob>>,
}

impl Source {
    pub fn picker(source: Self, config: picker::Config) -> Picker<Blob, Blob, Source> {
        Picker::builder().config(config).build().with_source(source)
    }
}

impl Populator<Blob, Blob, Data<Blob>> for Source {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn update_config(&mut self, config: Blob) {
        self.config = config;
    }

    fn build_injector(&self) -> crate::injector::FinderFn<Data<Blob>> {
        let entries = self.results.clone();
        Arc::new(move |tx| {
            entries.par_iter().for_each(|entry| {
                let _ = tx.send(entry.clone());
            });
            Ok(())
        })
    }
}
