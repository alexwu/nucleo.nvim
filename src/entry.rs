use std::fmt::Debug;
use std::path::Path;

use mlua::{
    prelude::{Lua, LuaResult, LuaTable, LuaValue},
    FromLua, IntoLua, LuaSerdeExt, UserData, UserDataFields, UserDataMethods,
};
use nucleo::Utf32String;
use serde::{Deserialize, Serialize};

pub trait Entry:
    for<'a> Deserialize<'a> + for<'a> FromLua<'a> + Debug + Serialize + Clone + Sync + Send + 'static
{
    fn from_path(path: &Path, cwd: Option<String>) -> Self;

    fn display(&self) -> String;
    fn indices(&self) -> Vec<(u32, u32)>;
    fn is_selected(&self) -> bool;
    fn into_utf32(self) -> Utf32String;
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
    fn data(&self) -> LuaValue {
        LuaValue::Nil
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEntry {
    // display: String,
    value: String,
    // selected: bool,
    // indices: Vec<(u32, u32)>,
}

// impl Entry for CustomEntry {
//     fn from_path(path: &Path, cwd: Option<String>) -> Self {
//         todo!()
//     }
//
//     fn display(&self) -> String {
//         self.display.to_string()
//     }
//
//     fn indices(&self) -> Vec<(u32, u32)> {
//         self.indices.clone()
//     }
//
//     fn is_selected(&self) -> bool {
//         self.selected
//     }
//
//     fn into_utf32(self) -> Utf32String {
//         self.display.into()
//     }
//
//     fn with_indices(self, indices: Vec<(u32, u32)>) -> Self {
//         Self { indices, ..self }
//     }
//
//     fn with_selected(self, selected: bool) -> Self {
//         Self { selected, ..self }
//     }
// }

impl FromLua<'_> for CustomEntry {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;

        Ok(Self {
            // display: table.get("display")?,
            value: table.get("value")?,
            // selected: table.get("selected")?,
            // indices: vec![],
        })
    }
}
