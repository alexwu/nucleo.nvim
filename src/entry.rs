use std::{fmt::Debug, str::FromStr};

use mlua::{
    prelude::{Lua, LuaResult, LuaTable, LuaValue},
    ExternalResult, FromLua, IntoLua, LuaSerdeExt,
};
use serde::{Deserialize, Deserializer, Serialize};
use strum::{Display, EnumString};

use crate::{picker::Blob, previewer::PreviewOptions, sources::Populator};

pub trait Entry:
    for<'a> Deserialize<'a> + Debug + Serialize + Clone + Sync + Send + 'static
{
    fn display(&self) -> String;
    fn ordinal(&self) -> String;
    fn indices(&self) -> Vec<(u32, u32)>;
    fn is_selected(&self) -> bool;
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
}

#[derive(Debug, Clone, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum DataKind {
    File,
    String,
    #[strum(default)]
    Custom(String),
}

impl Serialize for DataKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DataKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DataKind::from_str(&s).expect("Strum should be defaulting here"))
    }
}

impl FromLua<'_> for DataKind {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua<'_> for DataKind {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue<'_>> {
        self.to_string().into_lua(lua)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data<T>
where
    T: Clone + Debug + Sync + Send + for<'a> Deserialize<'a> + 'static,
{
    pub display: String,
    pub ordinal: String,
    pub kind: DataKind,
    pub selected: bool,
    pub indices: Vec<(u32, u32)>,
    #[serde(
        bound = "T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static"
    )]
    pub value: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_options: Option<PreviewOptions>,
}

#[buildstructor::buildstructor]
impl<T> Data<T>
where
    T: Clone + Debug + Sync + Send + for<'a> Deserialize<'a>,
{
    #[builder]
    pub fn new<V: Into<String>>(
        kind: DataKind,
        display: V,
        ordinal: V,
        value: T,
        preview_options: Option<PreviewOptions>,
    ) -> Self {
        Self {
            kind,
            value,
            preview_options,
            display: display.into(),
            ordinal: ordinal.into(),
            selected: false,
            indices: vec![],
        }
    }

    pub fn with_preview_options(self, preview_options: PreviewOptions) -> Self {
        Self {
            preview_options: Some(preview_options),
            ..self
        }
    }
}

impl From<String> for Data<String> {
    fn from(value: String) -> Self {
        Self::new(DataKind::String, &value, &value, value.clone(), None)
    }
}

impl<T> FromLua<'_> for Data<T>
where
    T: Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + for<'a> FromLua<'a>
        + 'static,
{
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl<T> IntoLua<'_> for Data<T>
where
    T: Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + for<'a> FromLua<'a>
        + 'static,
{
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue<'_>> {
        lua.to_value(&self)
    }
}

impl<T> Entry for Data<T>
where
    T: Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        // + for<'a> FromLua<'a>
        + 'static,
{
    fn display(&self) -> String {
        self.display.clone()
    }

    fn ordinal(&self) -> String {
        self.ordinal.clone()
    }

    fn indices(&self) -> Vec<(u32, u32)> {
        self.indices.clone()
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self {
        Self { indices, ..self }
    }

    fn with_selected(self, selected: bool) -> Self {
        Self { selected, ..self }
    }
}
