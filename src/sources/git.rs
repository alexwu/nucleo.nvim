use std::{env::current_dir, path::Path, sync::Arc};

use crate::picker::{self, Blob, Data, DataKind, InjectorConfig, Picker};
use anyhow::bail;
use git2::Statuses;
use mlua::prelude::*;

use partially::Partial;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::files::{FinderFn, PreviewOptions};

#[derive(Debug, Clone, Serialize, Deserialize, Partial)]
#[partially(derive(Default, Debug))]
pub struct StatusConfig {
    cwd: String,
}

pub struct Repository(git2::Repository);

impl Repository {
    pub fn statuses(
        &self,
        options: Option<&mut git2::StatusOptions>,
    ) -> Result<FileStatuses<'_>, anyhow::Error> {
        match self.0.statuses(options) {
            Ok(statuses) => Ok(FileStatuses(statuses)),
            Err(err) => bail!(err),
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        match git2::Repository::open(path) {
            Ok(repo) => Ok(Self(repo)),
            Err(err) => bail!(err),
        }
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
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
    type Item = <git2::StatusIter<'a> as Iterator>::Item;

    fn next(&mut self) -> Option<git2::StatusEntry<'a>> {
        <git2::StatusIter as Iterator>::next(&mut self.0)
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

impl InjectorConfig for StatusConfig {}
impl FromLua<'_> for StatusConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let config: PartialStatusConfig = FromLua::from_lua(value, lua)?;
        Ok(config.into())
    }
}

pub fn call_or_get<T>(lua: &Lua, val: LuaValue, field: &str) -> LuaResult<T>
where
    T: for<'a> IntoLua<'a> + for<'a> FromLua<'a> + for<'a> Deserialize<'a>,
{
    let table = LuaTable::from_lua(val, lua)?;
    match table.get(field)? {
        LuaValue::Function(func) => {
            log::info!("in the function section");
            func.call::<_, T>(())
        }
        val => {
            log::info!("val: {:?}", &val);
            lua.from_value(val)
        }
    }
}

impl FromLua<'_> for PartialStatusConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let cwd: Option<String> = call_or_get(lua, value, "cwd")?;
        log::info!("{:?}", &cwd);

        Ok(PartialStatusConfig { cwd })
    }
}

impl FromLua<'_> for StatusEntry {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
impl From<StatusEntry> for Data<StatusEntry, PreviewOptions> {
    fn from(value: StatusEntry) -> Self {
        let file_path = value.path().expect("Invalid utf8");
        let path = Path::new(&file_path);
        let file_type = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let preview_options = PreviewOptions::builder()
            .line_start(0)
            .col_start(0)
            .file_type(file_type)
            .build();

        Data::new(
            DataKind::File,
            path.to_str().expect("Failed to convert path to string"),
            path.to_str().expect("Failed to convert path to string"),
            value.clone(),
            Some(preview_options),
        )
    }
}

pub fn injector(config: Option<StatusConfig>) -> FinderFn<StatusEntry, PreviewOptions> {
    let repo = Repository::open(config.unwrap_or_default().cwd).expect("Unable to open repository");

    Arc::new(move |tx| {
        let status_options = &mut git2::StatusOptions::new();
        status_options
            .include_ignored(false)
            .include_untracked(true);
        let statuses = repo
            .statuses(Some(status_options))
            .expect("Unable to get statuses");
        statuses.iter().for_each(|entry| {
            let entry: StatusEntry = entry.into();
            let data = Data::from(entry);
            log::info!("{:?}", &data);
            let _ = tx.send(data);
        });
    })
}

pub fn create_picker(
    file_options: Option<PartialStatusConfig>,
) -> anyhow::Result<Picker<StatusEntry, PreviewOptions, StatusConfig>> {
    let config = match file_options {
        Some(config) => config,
        None => PartialStatusConfig::default(),
    };
    let populator = injector(Some(config.into()));
    let picker: Picker<StatusEntry, PreviewOptions, StatusConfig> =
        Picker::new(picker::Config::default())
            .with_populator(Arc::new(move |tx| {
                populator(tx);
            }))
            .with_injector(Arc::new(injector));

    anyhow::Ok(picker)
}
