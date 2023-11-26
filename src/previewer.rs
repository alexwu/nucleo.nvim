use std::fs::File;
use std::io::BufReader;

use mlua::{UserData, UserDataMethods};
use ropey::Rope;
use serde::{Deserialize, Serialize};

// TODO: Add caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Previewer {}

impl Previewer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn preview_file(&self, path: &str, start_line: usize, end_line: usize) -> String {
        log::info!("Previewing file {}", path);
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return String::new(),
        };
        let text = match Rope::from_reader(BufReader::new(file)) {
            Ok(rope) => rope,
            Err(_) => return String::new(),
        };
        let end_line = text.len_lines().min(end_line);
        let start_idx = text.line_to_char(start_line);
        let end_idx = text.line_to_char(end_line);

        text.slice(start_idx..end_idx).to_string()
    }
}
impl UserData for Previewer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "preview_file",
            |_lua, this, params: (Option<String>, usize, usize)| match params.0 {
                Some(path) => Ok(this.preview_file(&path, params.1, params.2)),
                None => Ok(String::new()),
            },
        );
    }
}
