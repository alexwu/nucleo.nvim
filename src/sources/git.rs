use std::{env::current_dir, path::Path, sync::Arc};

use buildstructor::Builder;
use git2::Statuses;
use mlua::prelude::*;
use partially::Partial;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use url::Url;

use super::{Populator, Sources};
use crate::{
    entry::{Data, DataKind},
    error::Result,
    injector::FinderFn,
    lua::call_or_get,
    picker::Picker,
    previewer::{PreviewKind, PreviewOptions},
};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Source {
    config: StatusConfig,
}

impl Populator<StatusEntry, StatusConfig, Data<StatusEntry>> for Source {
    fn name(&self) -> Sources {
        Sources::GitStatus
    }

    fn kind(&self) -> super::SourceKind {
        super::SourceKind::Rust
    }

    fn update_config(&mut self, config: StatusConfig) {
        self.config = config;
    }

    fn build_injector(&mut self, _: Option<&Lua>) -> FinderFn<Data<StatusEntry>> {
        let config = self.config.clone();
        let repo = Repository::open(config.cwd).expect("Unable to open repository");

        Arc::new(move |tx| {
            let status_options = &mut git2::StatusOptions::new();
            status_options
                .show(git2::StatusShow::Workdir)
                .update_index(true)
                .recurse_untracked_dirs(true)
                .include_ignored(false)
                .include_untracked(true);

            repo.statuses(Some(status_options))
                .expect("Unable to get statuses")
                .iter()
                .for_each(|entry| {
                    let entry: StatusEntry = entry;
                    let data = Data::from(entry);
                    let _ = tx.send(data);
                });

            Ok(())
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Partial)]
#[partially(derive(Default, Debug, Deserialize, Serialize))]
pub struct StatusConfig {
    cwd: String,
}

pub struct Repository(git2::Repository);

impl Repository {
    pub fn statuses(&self, options: Option<&mut git2::StatusOptions>) -> Result<FileStatuses<'_>> {
        Ok(self.0.statuses(options).map(FileStatuses)?)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(git2::Repository::discover(path).map(Self)?)
    }

    pub fn diff_index_to_workdir(
        &self,
        index: Option<&git2::Index>,
        opts: Option<&mut git2::DiffOptions>,
    ) -> Result<git2::Diff<'_>> {
        Ok(self.0.diff_index_to_workdir(index, opts)?)
    }
}

pub struct FileStatuses<'a>(Statuses<'a>);
pub struct StatusIter<'a>(git2::StatusIter<'a>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    path: Option<String>,
    path_bytes: Vec<u8>,
    status: Status,
}

impl StatusEntry {
    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn path(&self) -> Option<String> {
        self.path.clone()
    }
}

impl<'a> From<git2::StatusEntry<'a>> for StatusEntry {
    fn from(value: git2::StatusEntry<'a>) -> Self {
        Self {
            path: value.path().map(|v| v.to_string()),
            path_bytes: value.path_bytes().to_vec(),
            status: value.status().into(),
        }
    }
}

// Credit: https://github.com/extrawurst/gitui/blob/master/asyncgit/src/sync/status.rs
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize, EnumIs)]
pub enum Status {
    ///
    New,
    ///
    Modified,
    ///
    Deleted,
    ///
    Renamed,
    ///
    Typechange,
    ///
    Conflicted,
}

impl From<git2::Status> for Status {
    fn from(s: git2::Status) -> Self {
        if s.is_index_new() || s.is_wt_new() {
            Self::New
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            Self::Deleted
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            Self::Renamed
        } else if s.is_index_typechange() || s.is_wt_typechange() {
            Self::Typechange
        } else if s.is_conflicted() {
            Self::Conflicted
        } else {
            Self::Modified
        }
    }
}

impl<'a> Iterator for StatusIter<'a> {
    type Item = StatusEntry;

    fn next(&mut self) -> Option<StatusEntry> {
        <git2::StatusIter as Iterator>::next(&mut self.0).map(|iter| iter.into())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        <git2::StatusIter as Iterator>::size_hint(&self.0)
    }
}

impl<'a> FileStatuses<'a> {
    fn iter(&self) -> StatusIter {
        StatusIter(self.0.iter())
    }
}

unsafe impl Send for Repository {}
unsafe impl Sync for Repository {}
unsafe impl<'a> Send for FileStatuses<'a> {}
unsafe impl<'a> Sync for FileStatuses<'a> {}
unsafe impl<'a> Send for StatusIter<'a> {}
unsafe impl<'a> Sync for StatusIter<'a> {}

impl Default for StatusConfig {
    fn default() -> Self {
        let cwd = current_dir()
            .expect("Unable to get current directory")
            .to_string_lossy()
            .to_string();

        Self { cwd }
    }
}
impl From<PartialStatusConfig> for StatusConfig {
    fn from(value: PartialStatusConfig) -> Self {
        let mut config = StatusConfig::default();
        config.apply_some(value);
        config
    }
}

impl FromLua for StatusConfig {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        let config: PartialStatusConfig = FromLua::from_lua(value, lua)?;
        Ok(config.into())
    }
}

impl FromLua for PartialStatusConfig {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        let cwd: Option<String> = call_or_get(lua, value, "cwd")?;

        Ok(PartialStatusConfig { cwd })
    }
}

impl FromLua for StatusEntry {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
impl From<StatusEntry> for Data<StatusEntry> {
    fn from(value: StatusEntry) -> Self {
        let file_path = value.path().expect("Invalid utf8");
        let path = Path::new(&file_path);
        let file_extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let preview_kind = if value.status().is_new() {
            PreviewKind::File
        } else {
            PreviewKind::Diff
        };

        let full_path = path.canonicalize().ok();
        let uri = full_path.and_then(|fpath| Url::from_file_path(fpath).ok());
        let preview_options = PreviewOptions::builder()
            .kind(preview_kind)
            .line_start(0)
            .col_start(0)
            .and_uri(uri)
            .path(path.display().to_string())
            .file_extension(file_extension)
            .build();

        Data::new(
            DataKind::File,
            path.to_str().expect("Failed to convert path to string"),
            value.clone(),
            Some(0),
            Some(preview_options),
        )
    }
}

pub fn create_picker(
    file_options: Option<PartialStatusConfig>,
) -> Result<Picker<StatusEntry, StatusConfig, Source>> {
    let config = file_options.unwrap_or_default();

    let source = Source::builder().config(config).build();
    let picker: Picker<StatusEntry, StatusConfig, Source> =
        Picker::builder().source(source).build();

    Ok(picker)
}
