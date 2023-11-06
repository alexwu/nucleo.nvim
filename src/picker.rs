use std::cmp::{max, min};
use std::env::current_dir;
use std::path::Path;
use std::sync::Arc;

use mlua::{LuaSerdeExt, UserData, UserDataFields, UserDataMethods};
use nucleo::pattern::CaseMatching;
use nucleo::{Config, Nucleo, Utf32String};
use serde::{Deserialize, Serialize};

use crate::injector::Injector;

pub trait Entry: Serialize + Clone + Sync + Send + 'static {
    fn into_utf32(self) -> Utf32String;
    fn from_path(path: &Path, cwd: Option<String>) -> Self;
}

pub struct Matcher<T: Entry>(pub Nucleo<T>);
pub struct Status(pub nucleo::Status);

impl<T: Entry> Matcher<T> {
    pub fn pattern(&mut self) -> &mut nucleo::pattern::MultiPattern {
        &mut self.0.pattern
    }

    pub fn injector(&mut self) -> Injector<T> {
        self.0.injector().into()
    }

    pub fn tick(&mut self, timeout: u64) -> Status {
        Status(self.0.tick(timeout))
    }

    pub fn snapshot(&self) -> &nucleo::Snapshot<T> {
        self.0.snapshot()
    }
}

impl UserData for Status {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("changed", |_, this| Ok(this.0.changed));
        fields.add_field_method_get("running", |_, this| Ok(this.0.running));
    }
}

impl<T: Entry> From<Nucleo<T>> for Matcher<T> {
    fn from(value: Nucleo<T>) -> Self {
        Matcher(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub file_type: String,
}

impl Entry for FileEntry {
    fn into_utf32(self) -> Utf32String {
        self.path.into()
    }

    fn from_path(path: &Path, cwd: Option<String>) -> FileEntry {
        let val = path
            .strip_prefix(&cwd.unwrap_or_default())
            .expect("Failed to strip prefix")
            .to_str()
            .expect("Failed to convert path to string")
            .to_string();

        Self {
            path: val,
            file_type: path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }
}

pub struct Picker<T: Entry> {
    pub matcher: Matcher<T>,
    previous_query: String,
    cwd: String,
    selection_index: u32,
    lower_bound: u32,
    upper_bound: u32,
}

impl<T: Entry> Picker<T> {
    pub fn new(cwd: String) -> Self {
        fn notify() {}
        let matcher: Matcher<T> = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1).into();

        Self {
            matcher,
            cwd,
            selection_index: 0,
            lower_bound: 0,
            upper_bound: 50,
            previous_query: String::new(),
        }
    }

    pub fn upper_bound(&self) -> u32 {
        min(
            self.upper_bound,
            self.matcher.snapshot().matched_item_count(),
        )
    }

    pub fn lower_bound(&self) -> u32 {
        max(self.lower_bound, 0)
    }

    pub fn update_cursor(&mut self) {
        self.selection_index = self
            .selection_index
            .clamp(self.lower_bound(), self.upper_bound() - 1);
    }

    pub fn update_query(&mut self, query: String) {
        let previous_query = self.previous_query.clone();
        if query != previous_query {
            self.matcher.pattern().reparse(
                0,
                &query,
                CaseMatching::Smart,
                query.starts_with(&previous_query),
            );
            self.previous_query = query.to_string();
        }
    }

    pub fn move_cursor(&mut self, direction: Movement, change: u32) {
        log::info!("Moving cursor {:?} by {}", direction, change);
        log::info!("Lower bound: {}", self.lower_bound());
        log::info!("Upper bound: {}", self.upper_bound());
        let next_index = match direction {
            Movement::Up => self.selection_index + change,
            Movement::Down => {
                if change > self.selection_index {
                    0
                } else {
                    self.selection_index - change
                }
            }
        };

        self.selection_index = next_index;
        self.update_cursor();
        log::info!("Selection index: {}", self.selection_index);
    }

    pub fn current_matches(&self) -> Vec<T> {
        let snapshot = self.matcher.snapshot();

        let lower_bound = self.lower_bound();
        let upper_bound = self.upper_bound();

        Vec::from_iter(
            snapshot
                .matched_items(lower_bound..upper_bound)
                .map(|item| item.data.clone()),
        )
    }

    pub fn restart(&mut self) {
        self.matcher.0.restart(true)
    }

    pub fn populate_files(&mut self) {
        let dir = current_dir().unwrap();
        let injector = self.matcher.injector();
        std::thread::spawn(move || {
            injector.populate_files(dir.to_string_lossy().to_string(), true);
        });
    }
}

impl<T: Entry> Default for Picker<T> {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

impl<T: Entry> UserData for Picker<T> {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("update_query", |_lua, this, params: (String,)| {
            this.update_query(params.0);
            Ok(())
        });

        methods.add_method_mut("move_cursor_up", |_lua, this, ()| {
            this.move_cursor(Movement::Down, 1);
            Ok(())
        });

        methods.add_method_mut("move_cursor_down", |_lua, this, ()| {
            this.move_cursor(Movement::Up, 1);
            Ok(())
        });

        methods.add_method("current_matches", |lua, this, ()| {
            Ok(lua.to_value(&this.current_matches()))
        });

        methods.add_method("get_selection_index", |_lua, this, ()| {
            Ok(this.selection_index)
        });

        methods.add_method("get_selection", |lua, this, ()| {
            match this
                .matcher
                .snapshot()
                .get_matched_item(this.selection_index)
            {
                Some(selection) => Ok(lua.to_value(selection.data)),
                None => Err(mlua::Error::runtime(std::format!(
                    "Failed getting the selection at selection_index: {}",
                    this.selection_index
                ))),
            }
        });

        methods.add_method_mut("tick", |_lua, this, ms: u64| Ok(this.matcher.tick(ms)));

        methods.add_method_mut("populate_files", |_lua, this, _params: ()| {
            this.populate_files();
            Ok(())
        });

        methods.add_method_mut("restart", |_lua, this, _params: ()| {
            this.restart();
            Ok(())
        });
    }
}
