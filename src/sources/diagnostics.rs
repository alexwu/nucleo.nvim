use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::picker::{self, Blob, Data, Picker, Previewable};

use super::files::PreviewOptions;

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
}

impl Diagnostic {
    pub fn from_diagnostic(data: Diagnostic) -> Data<Diagnostic, Blob> {
        let message = data.message.clone().replace('\n', " ");
        Data::new(message, data, None)
    }
}

impl FromLua<'_> for Diagnostic {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        // let table = LuaTable::from_lua(value, lua)?;
        // Ok(Self {
        //     lnum: table.get("lnum")?,
        //     col: table.get("col")?,
        //     message: table.get("message")?,
        //     end_col: table.get("end_col")?,
        //     end_lnum: table.get("end_lnum")?,
        //     source: table.get("source")?,
        //     code: table.get("code")?,
        //     bufnr: table.get("bufnr")?,
        // })
        lua.from_value(value)
    }
}

impl<'lua> IntoLua<'lua> for Diagnostic {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        lua.to_value_with(
            &self,
            LuaSerializeOptions::default().serialize_none_to_null(false),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPreviewOptions {}
impl FromLua<'_> for DiagnosticPreviewOptions {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
impl Previewable for DiagnosticPreviewOptions {}

impl From<Diagnostic> for Data<Diagnostic, PreviewOptions> {
    fn from(value: Diagnostic) -> Self {
        let message = value.message.clone().replace('\n', " ");
        log::info!("{:?}", &value);
        let preview_options = PreviewOptions::builder()
            .line_start(value.lnum)
            .and_line_end(value.end_lnum)
            .col_start(value.col)
            .and_col_end(value.end_col)
            .and_bufnr(value.bufnr)
            .build();
        Data::new(message, value, Some(preview_options))
    }
}

pub fn create_picker() -> anyhow::Result<Picker<Diagnostic, PreviewOptions>> {
    anyhow::Ok(Picker::new(picker::Config::default()))
}
