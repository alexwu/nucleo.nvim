use std::{fmt::Debug, str::FromStr, sync::Arc};

use mlua::{prelude::*, FromLua, IntoLua, LuaSerdeExt};
use serde::{Deserialize, Deserializer, Serialize};
use strum::{Display, EnumString};

use crate::previewer::PreviewOptions;

pub trait Ordinal {
    fn ordinal(&self) -> String;
}

pub trait Entry: IntoLua + Debug + FromLua + Sync + Send + 'static {
    fn display(&self) -> &str;
    fn ordinal(&self) -> &str;
    fn indices(&self) -> Vec<(u32, u32)>;
    fn is_selected(&self) -> bool;
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn set_indices(&mut self, indices: Vec<(u32, u32)>);
    fn with_selected(self, selected: bool) -> Self;
    fn set_selected(&mut self, selected: bool);
}

impl<T: Entry> Ordinal for T {
    fn ordinal(&self) -> String {
        todo!()
    }
}

pub trait IntoUtf32String {
    fn into_utf32_string(&self) -> crate::nucleo::Utf32String;
}

impl<T: Entry> IntoUtf32String for T {
    fn into_utf32_string(&self) -> crate::nucleo::Utf32String {
        self.ordinal().into()
    }
}

pub trait IntoData<T>
where
    T: Debug + IntoLua + FromLua + 'static,
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

impl FromLua for DataKind {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for DataKind {
    fn into_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue> {
        self.to_string().into_lua(lua)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data<T>
where
    T: Debug + IntoLua + FromLua,
{
    pub ordinal: String,
    pub score: u32,
    pub kind: DataKind,
    pub selected: bool,
    pub indices: Vec<(u32, u32)>,
    #[serde(bound = "T: Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static")]
    pub value: Arc<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_options: Option<PreviewOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickerData {}

impl<T> Clone for Data<T>
where
    T: Debug + IntoLua + FromLua,
    // T: Debug + Serialize + for<'a> Deserialize<'a>,
{
    fn clone(&self) -> Self {
        Self {
            ordinal: self.ordinal.clone(),
            score: self.score,
            kind: self.kind.clone(),
            selected: self.selected,
            indices: self.indices.clone(),
            value: self.value.clone(),
            preview_options: self.preview_options.clone(),
        }
    }
}

impl<T: Eq> Eq for Data<T> where T: Debug + IntoLua + FromLua + 'static {}

impl<T: Ord> Ord for Data<T>
where
    T: Debug + Ord + FromLua + IntoLua + 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

#[buildstructor::buildstructor]
impl<T> Data<T>
where
    T: Debug + FromLua + IntoLua + 'static,
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
            value: Arc::new(value),
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
    T: Debug + IntoLua + FromLua + 'static,
{
    fn score(&self) -> u32 {
        self.score
    }
}

impl<T> PartialEq for Data<T>
where
    T: Debug + IntoLua + FromLua + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl<T> PartialOrd for Data<T>
where
    T: Debug + IntoLua + FromLua + 'static,
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

impl<T> FromLua for Data<T>
where
    T: Debug + Sync + Send + IntoLua + FromLua + 'static,
{
    fn from_lua(value: LuaValue, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(Data {
            ordinal: table.get("ordinal")?,
            indices: lua.from_value(table.get("indices")?)?,
            score: table.get("score")?,
            kind: table.get("kind")?,
            selected: table.get("selected")?,
            preview_options: table.get("preview_options")?,
            value: table.get::<&str, T>("value")?.into(),
        })
    }
}

impl<T> IntoLua for Data<T>
where
    T: Debug + Sync + Send + IntoLua + FromLua + Serialize + 'static,
{
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("ordinal", self.ordinal)?;
        table.set("indices", lua.to_value(&self.indices)?)?;
        table.set("score", self.score)?;
        table.set("kind", self.kind)?;
        table.set("selected", self.selected)?;
        table.set("preview_options", self.preview_options)?;

        let value = lua.to_value(&self.value)?;
        table.set("value", value)?;

        table.into_lua(lua)
    }
}

impl<T> Entry for Data<T>
where
    T: Debug + Sync + Send + IntoLua + FromLua + Serialize + 'static,
{
    fn display(&self) -> &str {
        &self.ordinal
    }

    fn ordinal(&self) -> &str {
        &self.ordinal
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

    fn set_indices(&mut self, indices: Vec<(u32, u32)>) {
        self.indices = indices
    }

    fn with_selected(self, selected: bool) -> Self {
        Self { selected, ..self }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected
    }
}
