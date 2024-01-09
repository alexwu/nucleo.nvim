use std::{env::current_dir, path::Path, sync::Arc};

use anyhow::bail;
use buildstructor::Builder;
use git2::{DiffDelta, Statuses};
use mlua::prelude::*;
use partially::Partial;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use url::Url;

use super::{git::Repository, Populator, Sources};
use crate::{
    entry::{Data, DataKind},
    injector::{FinderFn, FromPartial},
    picker::Picker,
    previewer::{PreviewKind, PreviewOptions},
    sources::git::call_or_get,
};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Source {
    config: HunkConfig,
}

impl Source {
    pub fn picker(
        options: Option<PartialHunkConfig>,
    ) -> anyhow::Result<Picker<Hunk, HunkConfig, Source>> {
        let config = match options {
            Some(config) => config,
            None => PartialHunkConfig::default(),
        };
        let source = Source::builder().config(config).build();
        let picker: Picker<Hunk, HunkConfig, Source> =
            Picker::builder().multi_sort(false).source(source).build();

        anyhow::Ok(picker)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Hunk {}

fn file_cb(delta: DiffDelta, _: f32) -> bool {
    log::info!("delta {:?}", delta);
    true
}

impl Populator<Hunk, HunkConfig, Data<Hunk>> for Source {
    fn name(&self) -> Sources {
        Sources::GitHunks
    }

    fn kind(&self) -> super::SourceKind {
        super::SourceKind::Rust
    }

    fn update_config(&mut self, config: HunkConfig) {
        self.config = config;
    }

    fn build_injector(&self, _: Option<&Lua>) -> FinderFn<Data<Hunk>> {
        let config = self.config.clone();
        let repo = Repository::open(config.cwd).expect("Unable to open repository");

        Arc::new(move |tx| {
            // let status_options = &mut git2::StatusOptions::new();
            // status_options
            //     .show(git2::StatusShow::Workdir)
            //     .update_index(true)
            //     .recurse_untracked_dirs(true)
            //     .include_ignored(false)
            //     .include_untracked(true);
            //
            repo.diff_index_to_workdir(None, None)?
                .foreach(&mut file_cb, None, None, None)?;
            // repo.statuses(Some(status_options))
            //     .expect("Unable to get statuses")
            //     .iter()
            //     .for_each(|entry| {
            //         let entry: StatusEntry = entry;
            //         let data = Data::from(entry);
            //         log::info!("{:?}", &data);
            //         let _ = tx.send(data);
            //     });

            Ok(())
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial, FromLua)]
#[partially(derive(Default, Debug, Deserialize, Serialize))]
pub struct HunkConfig {
    cwd: String,
}

impl From<PartialHunkConfig> for HunkConfig {
    fn from(value: PartialHunkConfig) -> Self {
        HunkConfig::from_partial(value)
    }
}

impl FromLua<'_> for PartialHunkConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let cwd: Option<String> = call_or_get(lua, value, "cwd")?;
        log::info!("{:?}", &cwd);

        Ok(Self { cwd })
    }
}

impl Default for HunkConfig {
    fn default() -> Self {
        let cwd = current_dir()
            .expect("Unable to get current directory")
            .to_string_lossy()
            .to_string();

        Self { cwd }
    }
}
