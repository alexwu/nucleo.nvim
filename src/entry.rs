use std::fmt::Debug;

use mlua::{
    prelude::{Lua, LuaResult, LuaTable, LuaValue},
    FromLua, IntoLua, LuaSerdeExt, UserData, UserDataFields, UserDataMethods,
};
use nucleo::Utf32String;
use serde::{Deserialize, Serialize};

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
    pub value: String,
}

impl FromLua<'_> for CustomEntry {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;

        Ok(Self {
            value: table.get("value")?,
        })
    }
}
