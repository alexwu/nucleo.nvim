use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use crossbeam_channel::bounded;
use mlua::{
    prelude::{Lua, LuaResult, LuaTable, LuaValue},
    FromLua, LuaSerdeExt, UserData, UserDataFields, UserDataMethods,
};
use nucleo::pattern::CaseMatching;
use nucleo::{Nucleo, Utf32String};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use range_rover::range_rover;
use serde::{Deserialize, Serialize};

use crate::buffer::{BufferContents, Contents, Cursor, Relative, Window};
use crate::injector::Injector;

pub trait Entry: Serialize + Clone + Sync + Send + 'static {
    fn into_utf32(self) -> Utf32String;
    fn from_path(path: &Path, cwd: Option<String>) -> Self;
    fn set_selected(&mut self, selected: bool);
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
}

pub struct Matcher<T: Entry>(pub Nucleo<T>);

#[derive(Default)]
pub struct StringMatcher(pub nucleo::Matcher);

pub static STRING_MATCHER: Lazy<Arc<Mutex<StringMatcher>>> =
    Lazy::new(|| Arc::new(Mutex::new(StringMatcher::default())));

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

impl From<nucleo::Matcher> for StringMatcher {
    fn from(value: nucleo::Matcher) -> Self {
        StringMatcher(value)
    }
}

impl From<StringMatcher> for nucleo::Matcher {
    fn from(val: StringMatcher) -> Self {
        val.0
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
    pub match_value: String,
    pub file_type: String,
    pub selected: bool,
    pub indices: Vec<(u32, u32)>,
}

impl Entry for FileEntry {
    fn into_utf32(self) -> Utf32String {
        self.match_value.into()
    }

    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self {
        Self { indices, ..self }
    }

    fn from_path(path: &Path, cwd: Option<String>) -> FileEntry {
        let full_path = path.to_str().expect("Failed to convert path to string");
        let match_value = path
            .strip_prefix(&cwd.unwrap_or_default())
            .expect("Failed to strip prefix")
            .to_str()
            .expect("Failed to convert path to string")
            .to_string();

        Self {
            selected: false,
            match_value,
            path: full_path.to_string(),
            indices: Vec::new(),
            file_type: path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn with_selected(self, selected: bool) -> Self {
        Self { selected, ..self }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

impl<T: Entry> Contents for Matcher<T> {
    fn len(&self) -> usize {
        self.0.snapshot().matched_item_count() as usize
    }
}

pub struct Picker<T: Entry> {
    pub matcher: Matcher<T>,
    pub string_matcher: StringMatcher,
    previous_query: String,
    cwd: String,
    cursor: Cursor,
    window: Window,
    selections: BTreeSet<u32>,
    sender: crossbeam_channel::Sender<()>,
    receiver: crossbeam_channel::Receiver<()>,
    git_ignore: bool,
}

impl<T: Entry> Picker<T> {
    pub fn new(cwd: String) -> Self {
        let (sender, receiver) = bounded::<()>(1);
        let notifier = sender.clone();
        let notify = Arc::new(move || {
            if notifier.try_send(()).is_ok() {
                log::info!("Message sent!")
            };
        });
        let matcher: Matcher<T> = Nucleo::new(nucleo::Config::DEFAULT, notify, None, 1).into();
        let string_matcher = StringMatcher::default();

        Self {
            matcher,
            string_matcher,
            cwd,
            receiver,
            sender,
            git_ignore: true,
            cursor: Cursor::default(),
            previous_query: String::new(),
            selections: BTreeSet::new(),
            window: Window::new(50),
        }
    }

    pub fn tick(&mut self, timeout: u64) -> Status {
        let status = self.matcher.tick(timeout);
        if status.0.changed {
            self.update_cursor();
        }
        status
    }

    fn try_recv(&self) -> Result<(), crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn should_rerender(&self) -> bool {
        !self.receiver.is_empty()
    }

    pub fn total_matches(&self) -> u32 {
        self.matcher.snapshot().matched_item_count()
    }

    pub fn total_items(&self) -> u32 {
        self.matcher.snapshot().item_count()
    }

    pub fn lower_bound(&self) -> u32 {
        max(self.window().start() as u32, 0)
    }

    pub fn upper_bound(&self) -> u32 {
        min(self.window.end() as u32, self.total_matches())
    }

    pub fn update_cursor(&mut self) {
        self.set_cursor_pos(self.cursor.pos());
    }

    pub fn update_window(&mut self, height: u32) {
        log::info!("Setting upper bound to {}", &height);
        self.set_window_height(height.try_into().unwrap_or(usize::MAX));
    }

    pub fn update_query(&mut self, query: String) {
        log::info!("Updating query: {}", &query);
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

    pub fn update_cwd(&mut self, cwd: &str) {
        self.cwd = cwd.to_string();
    }

    pub fn move_cursor(&mut self, direction: Movement, change: u32) {
        log::info!("Moving cursor {:?} by {}", direction, change);
        self.tick(10);

        if self.total_matches() == 0 {
            return;
        }

        let last_window_pos = self.window().start();

        let new_pos = match direction {
            Movement::Up => {
                self.cursor.pos().saturating_add(change as usize) as u32 % self.total_matches()
            }
            Movement::Down => {
                self.cursor
                    .pos()
                    .saturating_add(self.total_matches() as usize)
                    .saturating_sub(change as usize) as u32
                    % self.total_matches()
            }
        };
        self.set_cursor_pos(new_pos as usize);

        if last_window_pos != self.window().start() {
            let _ = self.sender.try_send(());
        }

        log::info!("Selection index: {}", self.cursor.pos());
    }

    pub fn current_matches(&self) -> Vec<T> {
        let mut indices = Vec::new();
        let snapshot = self.matcher.snapshot();
        log::info!("Item count: {:?}", snapshot.item_count());
        log::info!("Match count: {:?}", snapshot.matched_item_count());
        let string_matcher = &mut STRING_MATCHER.lock().0;

        // FIXME: If the window start is greater than the upper bownd (if it falls back to toal matches),
        let lower_bound = self.window().start() as u32;
        let upper_bound = self.window().end().min(self.total_matches() as usize) as u32;

        snapshot
            .matched_items(lower_bound..upper_bound)
            .map(|item| {
                snapshot.pattern().column_pattern(0).indices(
                    item.matcher_columns[0].slice(..),
                    string_matcher,
                    &mut indices,
                );
                indices.sort_unstable();
                indices.dedup();

                let ranges = range_rover(indices.drain(..))
                    .into_iter()
                    .map(|range| range.into_inner());
                // TODO: Probably a better way to do this
                item.data.clone().with_indices(ranges.collect())
            })
            .collect::<Vec<_>>()
    }

    pub fn restart(&mut self) {
        self.matcher.0.restart(true);
    }

    pub fn populate_files(&mut self) {
        let dir = self.cwd.clone();
        let git_ignore = self.git_ignore;
        let injector = self.matcher.injector();
        std::thread::spawn(move || {
            injector.populate_files_sorted(dir, git_ignore);
        });
    }

    pub fn select(&mut self, index: u32) {
        self.selections.insert(index);
    }

    pub fn deselect(&mut self, index: u32) {
        self.selections.remove(&index);
    }

    pub fn selections(&self) -> Vec<T> {
        self.selections
            .iter()
            .filter_map(|selection| {
                self.matcher
                    .snapshot()
                    .get_matched_item(*selection)
                    .map(|item| item.data.clone())
            })
            .collect()
    }

    pub fn cursor_pos(&self) -> Option<u32> {
        if self.total_matches() == 0 {
            None
        } else {
            self.get_cursor_pos(Relative::Window).try_into().ok()
        }
    }
}

impl<T: Entry> Default for Picker<T> {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl<T: Entry> Contents for Picker<T> {
    fn len(&self) -> usize {
        self.total_matches().try_into().unwrap_or(usize::MAX)
    }
}

impl<T: Entry> BufferContents<T> for Picker<T> {
    fn lines(&self) -> Vec<T> {
        self.current_matches()
    }

    fn window(&self) -> &crate::buffer::Window {
        &self.window
    }

    fn window_mut(&mut self) -> &mut crate::buffer::Window {
        &mut self.window
    }

    fn cursor(&self) -> &crate::buffer::Cursor {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut crate::buffer::Cursor {
        &mut self.cursor
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub cwd: Option<String>,
}

impl FromLua<'_> for Config {
    fn from_lua(value: LuaValue<'_>, lua: &'_ Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(value, lua)?;
        Ok(Config {
            cwd: table.get("cwd")?,
        })
    }
}

impl<T: Entry> UserData for Picker<T> {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("update_query", |_lua, this, params: (String,)| {
            this.update_query(params.0);
            Ok(())
        });

        methods.add_method_mut("update_cwd", |_lua, this, params: (String,)| {
            this.update_cwd(&params.0);
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

        methods.add_method_mut("update_window", |_lua, this, params: (usize,)| {
            this.update_window(params.0 as u32);
            Ok(())
        });

        methods.add_method("current_matches", |lua, this, ()| {
            Ok(lua.to_value(&this.current_matches()))
        });

        methods.add_method("total_items", |_lua, this, ()| Ok(this.total_items()));
        methods.add_method("total_matches", |_lua, this, ()| Ok(this.total_matches()));

        methods.add_method("get_selection_index", |_lua, this, ()| {
            Ok(this.get_cursor_pos(Relative::Window))
        });

        methods.add_method("get_cursor_pos", |_lua, this, ()| Ok(this.cursor_pos()));

        methods.add_method("get_selection", |lua, this, ()| {
            match this
                .matcher
                .snapshot()
                .get_matched_item(this.cursor.pos() as u32)
            {
                Some(selection) => Ok(lua.to_value(selection.data)),
                None => {
                    log::error!("Failed getting the selection at selection_index: {}, lower_bound: {}, upper_bound: {}", this.cursor.pos(), this.lower_bound(), this.upper_bound());
                    Err(mlua::Error::runtime(std::format!( "Failed getting the selection at selection_index: {}", this.cursor.pos() )))
                },
            }
        });

        methods.add_method_mut("tick", |_lua, this, ms: u64| Ok(this.tick(ms)));

        methods.add_method_mut("populate_files", |_lua, this, _params: ()| {
            this.populate_files();
            Ok(())
        });

        methods.add_method_mut("restart", |_lua, this, _params: ()| {
            this.restart();
            Ok(())
        });

        methods.add_method("should_rerender", |_lua, this, ()| {
            Ok(this.should_rerender())
        });

        methods.add_method("force_rerender", |_lua, this, ()| {
            let _ = this.sender.try_send(());
            Ok(())
        });

        methods.add_method("drain_channel", |_lua, this, ()| {
            let _ = this.receiver.try_recv();
            Ok(())
        });
    }
}
