use std::path::Path;
use std::{collections::HashMap, fs::File};
use std::{fmt::Debug, io::BufReader};

use buildstructor::Builder;
use ignore::WalkBuilder;
use mlua::prelude::*;
use mlua::UserData;
use mlua::UserDataMethods;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumIs, EnumString};

pub trait Previewable:
    Serialize + for<'a> FromLua<'a> + for<'a> Deserialize<'a> + Clone + Debug + Send + Sync + 'static
{
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Previewer {
    #[serde(skip)]
    file_cache: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display, EnumIs)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PreviewKind {
    #[default]
    Skip,
    File,
    Folder,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(default)]
#[derive(Default)]
pub struct PreviewOptions {
    pub kind: PreviewKind,
    pub line_start: usize,
    pub line_end: Option<usize>,
    pub col_start: usize,
    pub col_end: Option<usize>,
    pub bufnr: Option<usize>,
    pub path: Option<String>,
    pub uri: Option<String>,
    pub file_extension: Option<String>,
}

impl<'a> FromLua<'a> for PreviewOptions {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl<'a> IntoLua<'a> for PreviewOptions {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        lua.to_value_with(
            &self,
            LuaSerializeOptions::default().serialize_none_to_null(false),
        )
    }
}

impl Previewable for PreviewOptions {}

impl Previewer {
    pub fn new() -> Self {
        Self {
            file_cache: HashMap::new(),
        }
    }

    pub fn preview_file(&mut self, path: &str, start_line: usize, end_line: usize) -> String {
        log::info!("Previewing file {}", path);
        if let Some(contents) = self.file_cache.get(path) {
            log::info!("Using cached contents for {}", path);
            return contents.to_string();
        };
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return String::new(),
        };
        let text = match Rope::from_reader(BufReader::new(file)) {
            Ok(rope) => rope,
            Err(_) => return String::new(),
        };
        let end_line = text.len_lines().min(end_line.max(start_line));
        let start_idx = text.line_to_char(start_line);
        let end_idx = text.line_to_char(end_line);

        let content = text.slice(start_idx..end_idx).to_string();
        self.file_cache.insert(path.to_string(), content.clone());

        content
    }

    pub fn preview_folder(&mut self, path: &str) -> String {
        log::info!("Previewing folder {}", path);
        if let Some(contents) = self.file_cache.get(path) {
            log::info!("Using cached contents for {}", path);
            return contents.to_string();
        };

        let dir = Path::new(&path);
        let mut walk_builder = WalkBuilder::new(dir);
        walk_builder
            .hidden(false)
            .git_ignore(false)
            .ignore(true)
            .sort_by_file_name(std::cmp::Ord::cmp)
            .max_depth(Some(1));

        let results: Vec<String> =
            Vec::from_iter(walk_builder.build().filter_map(|entry| match entry {
                Ok(file) => Some(file.path().display().to_string()),
                Err(_) => None,
            }));

        let content = results.join("\n");
        self.file_cache.insert(path.to_string(), content.clone());

        content
    }
}

impl UserData for Previewer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "preview_file",
            |_lua, this, params: (Option<String>, usize, usize)| match params.0 {
                Some(path) => {
                    let preview: Vec<String> = this
                        .preview_file(&path, params.1, params.2)
                        .split('\n')
                        .map(Into::into)
                        .collect();
                    Ok(preview.clone())
                }
                None => Ok(Vec::new()),
            },
        );

        methods.add_method_mut("preview_folder", |_lua, this, params: (Option<String>,)| {
            match params.0 {
                Some(path) => {
                    let preview: Vec<String> = this
                        .preview_folder(&path)
                        .split('\n')
                        .map(Into::into)
                        .collect();
                    Ok(preview.clone())
                }
                None => Ok(Vec::new()),
            }
        });

        methods.add_method_mut("reset", |_lua, this, ()| {
            this.file_cache.clear();
            Ok(())
        });
    }
}
