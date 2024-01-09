use std::path::Path;
use std::process::Command;
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
use smol_str::SmolStr;
use strum::{Display, EnumIs, EnumString};

// FIX: Need to invalidate cache when the position changes.
// Maybe i should just cache the whole rope instead?
// Also need to add a max file size...
// Also probably need to break the cache up by picker type?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Previewer {
    #[serde(skip)]
    file_cache: HashMap<String, String>,
}

// TODO: Probably should have a placeholder when previewing is skipped
#[derive(
    Default, Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display, EnumIs, Eq,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PreviewKind {
    #[default]
    Skip,
    File,
    Folder,
    Diff,
}

// TODO: Rename to `Metadata` or something. It's not really exclusive to the previewer anymore
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default, PartialEq, Eq)]
#[serde(default)]
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
    pub file_size: Option<usize>,
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

impl Previewer {
    pub fn new() -> Self {
        Self {
            file_cache: HashMap::new(),
        }
    }

    // pub fn preview(&mut self, options: PreviewOptions) -> (String, usize) {
    //     match options.kind {
    //         PreviewKind::Skip => (String::new(), 0),
    //         PreviewKind::File => {
    //             self.preview_file(path, start_line, end_line)
    //         },
    //         PreviewKind::Folder => todo!(),
    //         PreviewKind::Diff => todo!(),
    //     }
    // }

    pub fn preview_file(
        &mut self,
        path: &str,
        start_line: usize,
        end_line: usize,
    ) -> (String, usize) {
        let offset = 10;
        let adjusted_start = start_line.saturating_sub(offset);
        let cache_key = format!("{}:{}:{}", path, adjusted_start, end_line);
        log::info!("Previewing file {}", cache_key);
        if let Some(contents) = self.file_cache.get(&cache_key) {
            log::info!("Using cached contents for {}", cache_key);
            return (contents.to_string(), adjusted_start);
        };
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return (String::new(), 0),
        };
        let text = match Rope::from_reader(BufReader::new(file)) {
            Ok(rope) => rope,
            Err(_) => return (String::new(), 0),
        };
        let end_line = text.len_lines().min(end_line.max(adjusted_start));
        let start_idx = text.line_to_char(adjusted_start);
        let end_idx = text.line_to_char(end_line);

        let content = text.slice(start_idx..end_idx).to_string();
        self.file_cache
            .insert(cache_key.to_string(), content.clone());

        (content, adjusted_start)
    }

    pub fn preview_diff(&mut self, path: &str) -> String {
        log::info!("Previewing folder {}", path);
        if let Some(contents) = self.file_cache.get(path) {
            log::info!("Using cached contents for {}", path);
            return contents.to_string();
        };

        let output = Command::new("git")
            .args(["--no-pager", "diff", "--", path])
            .output()
            .expect("Failed to execute git diff");

        match String::from_utf8(output.stdout) {
            Ok(s) => {
                self.file_cache.insert(path.to_string(), s.clone());
                s
            }
            Err(err) => {
                log::error!("Failed to get preview diff for: {}", path);
                log::error!("{:?}", err);
                String::new()
            }
        }
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
            |lua, this, params: (Option<String>, usize, usize)| match params.0 {
                Some(path) => {
                    let (result, start) = this.preview_file(&path, params.1, params.2);
                    let preview: Vec<SmolStr> = result.split('\n').map(Into::into).collect();
                    lua.to_value(&(preview, start))
                }
                None => LuaSerdeExt::to_value::<(Vec<SmolStr>, usize)>(lua, &(vec![], 0)),
            },
        );

        methods.add_method_mut(
            "preview_folder",
            |lua, this, params: (Option<String>,)| match params.0 {
                Some(path) => {
                    let preview: Vec<SmolStr> = this
                        .preview_folder(&path)
                        .split('\n')
                        .map(Into::into)
                        .collect();
                    lua.to_value(&preview)
                }
                None => LuaSerdeExt::to_value::<Vec<SmolStr>>(lua, &vec![]),
            },
        );

        methods.add_method_mut(
            "preview_diff",
            |lua, this, params: (Option<String>,)| match params.0 {
                Some(path) => {
                    let preview: Vec<SmolStr> = this
                        .preview_diff(&path)
                        .split('\n')
                        .map(Into::into)
                        .collect();
                    lua.to_value(&preview)
                }
                None => LuaSerdeExt::to_value::<Vec<SmolStr>>(lua, &vec![]),
            },
        );

        methods.add_method_mut("reset", |_lua, this, ()| {
            this.file_cache.clear();
            Ok(())
        });
    }
}
