use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::picker::{Data, Blob};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    lnum: usize,
    col: usize,
    message: String,
}

impl Diagnostic {
    pub fn from_diagnostic(data: Diagnostic) -> Data<Diagnostic, Blob> {
        let message = data.message.replace('\n', " ");
        Data::new(message, data, None)
    }
}

impl FromLua<'_> for Diagnostic {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(Self {
            lnum: table.get("lnum")?,
            col: table.get("col")?,
            message: table.get("message")?,
        })
    }
}
