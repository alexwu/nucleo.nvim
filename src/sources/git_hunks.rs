use std::{env::current_dir, path::PathBuf, sync::Arc};

use buildstructor::Builder;
use git2::DiffDelta;
use mlua::prelude::*;
use partially::Partial;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{git::Repository, Populator, Sources};
use crate::{
    entry::{Data, DataKind},
    error::Result,
    injector::{FinderFn, FromPartial},
    lua::call_or_get,
    picker::Picker,
    previewer::{PreviewKind, PreviewOptions},
};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Source {
    config: HunkConfig,
}

impl Source {
    pub fn picker(options: Option<PartialHunkConfig>) -> Result<Picker<Hunk, HunkConfig, Source>> {
        let config = match options {
            Some(config) => config,
            None => PartialHunkConfig::default(),
        };
        let source = Source::builder().config(config).build();
        let picker: Picker<Hunk, HunkConfig, Source> =
            Picker::builder().multi_sort(false).source(source).build();

        Ok(picker)
    }

    pub fn lua_picker(lua: &Lua, options: Option<LuaValue>) -> mlua::Result<LuaValue> {
        let opts: Option<PartialHunkConfig> = options.and_then(|c| lua.from_value(c).ok()?);
        Source::picker(opts).into_lua_err()?.into_lua(lua)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Hunk {
    path: PathBuf,
    old_start: usize,
    new_start: usize,
    old_lines: usize,
    new_lines: usize,
}

fn file_cb(_delta: DiffDelta, _: f32) -> bool {
    // log::info!("delta {:?}", delta);
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

    fn build_injector(&mut self, _: Option<&Lua>) -> FinderFn<Data<Hunk>> {
        let config = self.config.clone();
        let repo = Repository::open(config.cwd).expect("Unable to open repository");

        Arc::new(move |tx| {
            let mut diff_options = git2::DiffOptions::new();
            diff_options.context_lines(0);
            repo.diff_index_to_workdir(None, Some(&mut diff_options))?
                .foreach(
                    &mut file_cb,
                    None,
                    Some(&mut |delta, hunk| {
                        log::debug!(
                            "delta {:?}",
                            delta.new_file().path().map(|p| p.to_path_buf())
                        );
                        log::debug!("hunk {:?}", hunk);
                        if let Some(path) = delta.new_file().path() {
                            let entry = Hunk::builder()
                                .path(path.to_path_buf())
                                .old_start(hunk.old_start() as usize)
                                .new_start(hunk.new_start() as usize)
                                .old_lines(hunk.old_lines() as usize)
                                .new_lines(hunk.new_lines() as usize)
                                .build();

                            if tx.send(entry.into()).is_err() {
                                return false;
                            }
                        }
                        true
                    }),
                    None,
                )?;

            Ok(())
        })
    }
}

impl From<Hunk> for Data<Hunk> {
    fn from(value: Hunk) -> Self {
        let file_extension = value
            .path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let preview_kind = PreviewKind::File;

        let line_start = value.new_start.saturating_sub(1);
        let new_end = (line_start + value.new_lines).max(line_start);

        let full_path = value.path.canonicalize().ok();
        let uri = full_path.and_then(|fpath| Url::from_file_path(fpath).ok());
        let preview_options = PreviewOptions::builder()
            .kind(preview_kind)
            .line_start(line_start)
            .line_end(new_end)
            .col_start(0)
            .and_uri(uri)
            .path(value.path.display().to_string())
            .file_extension(file_extension)
            .build();

        let ordinal = format!("{}:{} {}", value.new_start, new_end, value.path.display());

        Data::builder()
            .kind(DataKind::File)
            .ordinal(ordinal)
            .value(value.clone())
            .preview_options(preview_options)
            .build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partially(derive(Default, Debug, Deserialize, Serialize))]
pub struct HunkConfig {
    cwd: String,
}

impl FromLua for HunkConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl From<PartialHunkConfig> for HunkConfig {
    fn from(value: PartialHunkConfig) -> Self {
        HunkConfig::from_partial(value)
    }
}

impl FromLua for PartialHunkConfig {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        let cwd: Option<String> = call_or_get(lua, value, "cwd")?;

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
