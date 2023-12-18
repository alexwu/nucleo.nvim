use std::io::BufReader;
use std::{collections::HashMap, fs::File};

use mlua::{UserData, UserDataMethods};
use ropey::Rope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Previewer {
    #[serde(skip)]
    file_cache: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewKind {
    File,
}

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
                    Ok(preview.to_owned())
                }
                None => Ok(Vec::new()),
            },
        );

        methods.add_method_mut("reset", |_lua, this, ()| {
            this.file_cache.clear();
            Ok(())
        });
    }
}
