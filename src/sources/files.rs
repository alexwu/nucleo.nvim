use std::fmt::Debug;
use std::sync::Arc;
use std::{env::current_dir, path::Path};

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use mlua::prelude::*;
use partially::Partial;
use serde::{Deserialize, Serialize};

use crate::injector::FinderFn;
use crate::picker::{self, Data, DataKind, InjectorConfig, Picker};
use crate::previewer::{PreviewOptions, PreviewKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub path: String,
    pub file_type: String,
}

impl Value {
    fn from_path(path: &Path, cwd: Option<String>) -> Data<Value, PreviewOptions> {
        let full_path = path.to_str().expect("Failed to convert path to string");
        let match_value = path
            .strip_prefix(&cwd.unwrap_or_default())
            .expect("Failed to strip prefix")
            .to_str()
            .expect("Failed to convert path to string")
            .to_string();

        let file_type = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let value = Self {
            path: full_path.to_string(),
            file_type: file_type.clone(),
        };

        let preview_options = PreviewOptions::builder()
            .kind(PreviewKind::File)
            .line_start(0)
            .col_start(0)
            .file_type(file_type)
            .build();

        Data {
            kind: DataKind::File,
            selected: false,
            indices: Vec::new(),
            value,
            preview_options: Some(preview_options),
            display: match_value.clone(),
            ordinal: match_value,
        }
    }
}

impl FromLua<'_> for Value {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(Self {
            path: table.get("path")?,
            file_type: table.get("file_type")?,
        })
    }
}

#[derive(Debug, Clone, Partial, Serialize, Deserialize)]
#[partially(derive(Default, Debug))]
pub struct FileConfig {
    pub cwd: String,
    pub git_ignore: bool,
    pub ignore: bool,
    pub hidden: bool,
}

impl<'a> FromLua<'a> for FileConfig {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        let val: PartialFileConfig = FromLua::from_lua(value, lua)?;
        Ok(val.into())
    }
}

impl InjectorConfig for FileConfig {}

impl Default for FileConfig {
    fn default() -> Self {
        let cwd = current_dir()
            .expect("Unable to get current directory")
            .to_string_lossy()
            .to_string();

        Self {
            cwd,
            git_ignore: true,
            ignore: true,
            hidden: false,
        }
    }
}

impl FromLua<'_> for PartialFileConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        let cwd = match table.get::<&str, LuaValue>("cwd") {
            Ok(val) => match val {
                LuaValue::String(cwd) => Some(cwd.to_string_lossy().to_string()),
                LuaValue::Function(thunk) => Some(thunk.call::<_, String>(())?),
                _ => None,
            },
            _ => None,
        };

        Ok(PartialFileConfig {
            cwd,
            git_ignore: table.get("git_ignore")?,
            ignore: table.get("ignore")?,
            hidden: table.get("hidden")?,
        })
    }
}

impl From<PartialFileConfig> for FileConfig {
    fn from(value: PartialFileConfig) -> Self {
        let mut config = FileConfig::default();
        config.apply_some(value);
        config
    }
}

pub fn injector(config: Option<FileConfig>) -> FinderFn<Value, PreviewOptions> {
    let FileConfig {
        cwd,
        hidden,
        git_ignore,
        ignore,
    } = config.unwrap_or_default();

    let dir = Path::new(&cwd);
    log::info!("Spawning sorted file searcher at: {:?}", &dir);
    let mut walk_builder = WalkBuilder::new(dir);
    walk_builder
        .hidden(hidden)
        .follow_links(true)
        .git_ignore(git_ignore)
        .ignore(ignore)
        .sort_by_file_name(std::cmp::Ord::cmp);

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

    Arc::new(move |tx| {
        for path in walk_builder.build() {
            let cwd = cwd.clone();
            match path {
                Ok(file) if file.path().is_file() => {
                    if tx
                        .send(Value::from_path(file.path(), Some(cwd.clone())))
                        .is_ok()
                    {
                        // log::info!("Sending {:?}", file.path());
                    }
                }
                _ => (),
            };
        }
    })
}

pub fn create_picker(
    file_options: Option<PartialFileConfig>,
) -> anyhow::Result<Picker<Value, PreviewOptions, FileConfig>> {
    let _config = match file_options {
        Some(config) => config,
        None => PartialFileConfig::default(),
    };
    let picker: Picker<Value, PreviewOptions, FileConfig> =
        Picker::new(picker::Config::default()).with_injector(Arc::new(injector));

    anyhow::Ok(picker)
}
