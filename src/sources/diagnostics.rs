use std::str::FromStr;
use std::{env::current_dir, path::Path, sync::Arc};

use buildstructor::Builder;
use mlua::prelude::*;
use mlua::{FromLua, Function, Lua, LuaSerdeExt, RegistryKey, Value};
use partially::Partial;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use url::Url;

use super::{Populator, SourceKind, Sources};
use crate::injector::FromPartial;
use crate::{
    entry::{Data, DataKind},
    picker::Picker,
    previewer::{PreviewKind, PreviewOptions},
};

#[derive(Debug, EnumString, Display, Clone, Copy, Serialize, Deserialize, FromLua, Default)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Document,
    Workspace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromLua, Default, Partial)]
#[partially(derive(Clone, Copy, Debug, Serialize, Deserialize, Default))]
pub struct Config {
    scope: Scope,
}

impl FromLua<'_> for PartialConfig {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        let scope = match table.get::<&str, LuaValue>("scope") {
            Ok(val) => {
                if let LuaValue::String(scope) = val {
                    Some(Scope::from_str(&scope.to_string_lossy()).into_lua_err()?)
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(PartialConfig { scope })
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
    finder: Option<Arc<RegistryKey>>,
}

impl FromLua<'_> for Source {
    fn from_lua(value: Value<'_>, lua: &Lua) -> LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("Source wasn't given a table!"))
            .into_lua_err()?;

        let registry_key = match table.get::<&str, LuaValue>("finder")? {
            LuaValue::Function(thunk) => lua.create_registry_value(thunk)?,
            _ => todo!("Failed to implement finder"),
        };

        let partial_config: PartialConfig = lua.from_value(table.get::<_, LuaValue>("config")?)?;
        log::info!("diagnostics partial config: {:?}", &partial_config);
        let config = Config::from_partial(partial_config);

        Ok(Source::builder()
            .config(config)
            .finder(Arc::new(registry_key))
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

    fn build_injector(&self, lua: Option<&Lua>) -> crate::injector::FinderFn<Data<Diagnostic>> {
        let key = self.finder.clone().expect("No registry key stored!");
        let finder = lua
            .expect("No Lua object given!")
            .registry_value::<Function>(&key)
            .expect("Remember to make it so these return results!");
        let bufnr = match self.config.scope {
            Scope::Document => Some(0),
            Scope::Workspace => None,
        };
        let results = finder.call::<(Option<usize>,), Value>((bufnr,));
        let entries = match results {
            Ok(entries) => lua
                .expect("No lua!")
                .from_value::<Vec<Diagnostic>>(entries)
                .expect("Error with diagnostics"),
            Err(error) => {
                log::error!("Errored calling finder fn: {}", error);
                Vec::new()
            }
        };

        Arc::new(move |tx| {
            entries.clone().into_iter().for_each(|entry| {
                let _ = tx.send(entry.into());
            });
            Ok(())
        })
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
    code: Option<String>,
    bufnr: Option<usize>,
    pub severity: Option<usize>,
    user_data: serde_json::Value,
}

impl FromLua<'_> for Diagnostic {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl<'lua> IntoLua<'lua> for Diagnostic {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        lua.to_value(&self)
    }
}

impl From<Diagnostic> for Data<Diagnostic> {
    fn from(value: Diagnostic) -> Self {
        let bufnr = value.bufnr.unwrap_or_default();
        let file_path: String = unsafe {
            let mut error = crate::nvim::Error::new();
            crate::nvim::nvim_buf_get_name(bufnr as i32, core::ptr::null_mut(), &mut error)
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
            value.code.clone().unwrap_or_default(),
            message.clone(),
            relative,
        );
        let uri = Url::from_file_path(path).expect("Unable to create uri");
        log::info!("{:?}", &value);
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
            .score(((5 - severity) * 100) as u32)
            .preview_options(preview_options)
            .build()
    }
}

pub fn create_picker(source: Source) -> anyhow::Result<Picker<Diagnostic, Config, Source>> {
    anyhow::Ok(Picker::builder().multi_sort(true).source(source).build())
}
