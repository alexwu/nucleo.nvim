use std::{fmt::Debug, str::FromStr};

use mlua::{prelude::*, FromLua, IntoLua, LuaSerdeExt};
use serde::{Deserialize, Deserializer, Serialize};
use strum::{Display, EnumString};

use crate::previewer::PreviewOptions;

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

pub trait IntoUtf32String {
    fn into_utf32_string(self) -> crate::nucleo::Utf32String;
}

impl<T: Entry> IntoUtf32String for T {
    fn into_utf32_string(self) -> crate::nucleo::Utf32String {
        self.ordinal().clone().into()
    }
}

pub trait IntoData<T>
where
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    fn into_data(self) -> Data<T>;
}

// impl<T:> IntoData<T> for T
// where
//     T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
// {
//     fn into_data(self) -> Data<T> {
//         todo!()
//     }
// }
pub trait Scored {
    fn score(&self) -> u32;
}

#[derive(Debug, Clone, EnumString, Display, PartialEq, Eq)]
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
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    pub ordinal: String,
    pub score: u32,
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

impl<T: Eq> Eq for Data<T> where T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static {}

impl<T: Ord> Ord for Data<T>
where
    T: Clone + Debug + Serialize + Ord + for<'a> Deserialize<'a> + 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

#[buildstructor::buildstructor]
impl<T> Data<T>
where
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    #[builder]
    pub fn new<V: Into<String>>(
        kind: DataKind,
        ordinal: V,
        value: T,
        score: Option<u32>,
        preview_options: Option<PreviewOptions>,
    ) -> Self {
        Self {
            kind,
            value,
            preview_options,
            score: score.unwrap_or(0),
            ordinal: ordinal.into(),
            selected: false,
            indices: vec![],
        }
    }
}

impl<T> Scored for Data<T>
where
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    fn score(&self) -> u32 {
        self.score
    }
}

impl<T> PartialEq for Data<T>
where
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl<T> PartialOrd for Data<T>
where
    T: Clone + Debug + Serialize + for<'a> Deserialize<'a> + 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.score().cmp(&other.score()))
    }
}

impl From<String> for Data<String> {
    fn from(value: String) -> Self {
        Self::new(DataKind::String, &value, value.clone(), Some(0), None)
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
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
{
    fn display(&self) -> String {
        self.ordinal.clone()
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
