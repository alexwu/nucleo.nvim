use std::fmt::Debug;

use mlua::{
    prelude::{Lua, LuaResult, LuaTable, LuaValue},
    ExternalResult, FromLua, IntoLua, LuaSerdeExt,
};
use serde::{Deserialize, Serialize};

use crate::picker::Blob;

pub trait Entry:
    for<'a> Deserialize<'a> + for<'a> FromLua<'a> + Debug + Serialize + Clone + Sync + Send + 'static
{
    fn display(&self) -> String;
    fn indices(&self) -> Vec<(u32, u32)>;
    fn is_selected(&self) -> bool;
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
    fn data(&self) -> LuaValue {
        LuaValue::Nil
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEntry {
    pub display: Option<String>,
    pub value: Blob,
}

impl FromLua<'_> for CustomEntry {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        let val: LuaValue = table.get("value")?;
        let json_str = serde_json::to_value(&val).into_lua_err()?;

        Ok(Self {
            display: table.get("display")?,
            value: Blob(json_str),
        })
    }
}

impl IntoLua<'_> for CustomEntry {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue<'_>> {
        let table = lua.create_table()?;
        let value: LuaValue = lua.to_value(&self.value)?;
        let display = self.display;

        table.set("display", display)?;
        table.set("value", value)?;
        lua.to_value(&table)
    }
}
