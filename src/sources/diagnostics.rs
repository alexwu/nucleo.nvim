use std::str::FromStr;
use std::{env::current_dir, path::Path, sync::Arc};

use buildstructor::Builder;
use mlua::prelude::*;
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Value};
use oxi_api::Buffer;
use partially::Partial;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use url::Url;

use super::{Populator, SourceKind, Sources};
use crate::config;
use crate::error::Result;
use crate::injector::FromPartial;
use crate::{
    entry::{Data, DataKind},
    picker::Picker,
    previewer::{PreviewKind, PreviewOptions},
};

#[derive(Debug, EnumString, Display, Clone, Copy, Serialize, Deserialize, Default)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Document,
    Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Partial)]
#[partially(derive(Clone, Debug, Serialize, Deserialize, Default))]
pub struct Config {
    scope: Scope,
    #[serde(flatten, default)]
    picker_config: config::PartialConfig,
}

impl FromLua for PartialConfig {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value.clone(), lua)?;
        let scope = match table.get::<LuaValue>("scope") {
            Ok(val) => {
                if let LuaValue::String(scope) = val {
                    Some(Scope::from_str(&scope.to_string_lossy()).into_lua_err()?)
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(PartialConfig {
            scope,
            picker_config: lua.from_value(value)?,
        })
    }
}

impl IntoLua for Config {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for Config {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl From<PartialConfig> for Config {
    fn from(value: PartialConfig) -> Self {
        Config::from_partial(value)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Source {
    config: Config,
    #[serde(skip)]
    // finder: Option<Arc<Function>>,
    finder: Vec<Diagnostic>,
}

impl FromLua for Source {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("Source wasn't given a table!"))
            .into_lua_err()?;

        // let finder: Vec<Diagnostic> = table.get::<Vec<Diagnostic>>("finder")?;

        let partial_config: PartialConfig = lua.from_value(table.get::<LuaValue>("config")?)?;
        log::debug!("diagnostics partial config: {:?}", &partial_config);
        let config = Config::from_partial(partial_config);
        let bufnr = match config.scope {
            Scope::Document => Some(0),
            Scope::Workspace => None,
        };

        let finder = match table.get::<LuaValue>("finder")? {
            LuaValue::Function(thunk) => thunk.call::<Vec<Diagnostic>>((bufnr,))?,
            LuaValue::Table(entries) => lua.from_value(mlua::Value::Table(entries))?,
            _ => todo!("Failed to implement finder"),
        };

        Ok(Source::builder()
            .config(config)
            // .finder(Arc::new(finder))
            .finder(finder)
            .build())
    }
}

impl Populator<Diagnostic, Config, Data<Diagnostic>> for Source {
    fn name(&self) -> Sources {
        Sources::Diagnostics
    }

    fn kind(&self) -> super::SourceKind {
        SourceKind::Lua
    }

    fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    fn build_injector(&mut self, lua: Option<&Lua>) -> crate::injector::FinderFn<Data<Diagnostic>> {
        // let finder = self.finder.clone().expect("No finder stored?");
        let entries = self.finder.clone();
        // let bufnr = match self.config.scope {
        //     Scope::Document => Some(0),
        //     Scope::Workspace => None,
        // };
        // let results = finder.call::<Value>((bufnr,));
        // let entries = match results {
        //     Ok(entries) => lua
        //         .expect("No lua!")
        //         .from_value::<Vec<Diagnostic>>(entries)
        //         .expect("Error with diagnostics"),
        //     Err(error) => {
        //         log::error!("Errored calling finder fn: {}", error);
        //         Vec::new()
        //     }
        // };

        Arc::new(move |tx| {
            entries.clone().into_iter().for_each(|entry| {
                let _ = tx.send(entry.into());
            });
            Ok(())
        })
    }
}

#[derive(Debug, Clone, Deserialize, Display)]
#[serde(untagged)]
enum Code {
    Integer(i64),
    #[strum(default)]
    String(String),
}

impl Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Code::String(inner) => serializer.serialize_str(inner),
            Code::Integer(inner) => serializer.serialize_i64(*inner),
        }
    }
}

impl IntoLua for Code {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Code::Integer(val) => Ok(LuaValue::Integer(val)),
            Code::String(val) => val.into_lua(lua),
        }
    }
}

impl FromLua for Code {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(integer) => Ok(Code::Integer(integer)),
            val => Ok(Code::String(val.to_string()?)),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub lnum: usize,
    pub col: usize,
    pub message: String,
    end_col: Option<usize>,
    end_lnum: Option<usize>,
    source: Option<String>,
    code: Option<Code>,
    bufnr: Option<usize>,
    pub severity: Option<usize>,
    user_data: serde_json::Value,
}

impl FromLua for Diagnostic {
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for Diagnostic {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        lua.to_value(&self)
    }
}

impl From<Diagnostic> for Data<Diagnostic> {
    fn from(value: Diagnostic) -> Self {
        let bufnr = value.bufnr.unwrap_or_default();
        let file_path: String = {
            let buffer = Buffer::from(bufnr as i32);
            buffer
                .get_name()
                .expect("Failed getting buffer name")
                .to_string_lossy()
                .clone()
                .to_string()
        };

        let dir = current_dir().expect("Unable to get current dir");
        let path = Path::new(&file_path);

        let file_extension = path.extension().map(|s| s.to_string_lossy().to_string());

        let relative = match path.strip_prefix(dir) {
            Ok(relative_path) => relative_path.display().to_string(),
            Err(err) => {
                log::error!("Unable to strip prefix on: {:?}", &path);
                log::error!("{:?}", err);

                path.display().to_string()
            }
        };

        let message = value.message.clone().replace('\n', " ");
        let ordinal = format!(
            "{} {} {}",
            value
                .code
                .clone()
                .unwrap_or_else(|| Code::String(String::new())),
            // value.code.clone().unwrap_or_default(),
            message.clone(),
            relative,
        );
        let uri = Url::from_file_path(path).expect("Unable to create uri");
        let preview_options = PreviewOptions::builder()
            .kind(PreviewKind::File)
            .path(path.display().to_string())
            .uri(uri)
            .line_start(value.lnum)
            .and_line_end(value.end_lnum)
            .col_start(value.col)
            .and_col_end(value.end_col)
            .and_bufnr(value.bufnr)
            .and_file_extension(file_extension)
            .build();

        let severity = value.severity.unwrap_or_default();

        Data::builder()
            .kind(DataKind::File)
            .ordinal(ordinal)
            .value(value)
            // Higher severities have lower values
            .score(((5 - severity) * 10) as u32)
            .preview_options(preview_options)
            .build()
    }
}

pub fn create_picker(source: Source) -> Result<Picker<Diagnostic, Config, Source>> {
    let picker_config = source.config.picker_config.clone();
    Ok(Picker::builder()
        .multi_sort(true)
        .source(source)
        .config(picker_config)
        .build())
}
