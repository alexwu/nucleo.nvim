use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::string::ToString;
use std::sync::Arc;

use buildstructor::{buildstructor, Builder};
use crossbeam_channel::bounded;
use mlua::ExternalResult;
use mlua::{prelude::Lua, LuaSerdeExt, UserData, UserDataMethods};
use nucleo_matcher::pattern::Pattern;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use partially::Partial;
use range_rover::range_rover;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use crate::buffer::{Buffer, Cursor, Relative};
use crate::config::{Config, PartialConfig, SortDirection};
use crate::entry::Scored;
use crate::error::Result;
use crate::nucleo::pattern::{CaseMatching, Normalization};
use crate::nucleo::{Nucleo, Status, Utf32Str};
use crate::previewer::PreviewOptions;
use crate::window::Window;

pub trait IntoUtf32String {
    fn into_utf32_string(self) -> crate::nucleo::Utf32String;
}

#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Sources {
    Files,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default)]
pub enum Location {
    Buffer(usize),
    File(PathBuf),
    #[default]
    None,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Default, PartialEq)]
pub struct Data {
    ordinal: String,
    location: Location,
    score: usize,
    preview_options: PreviewOptions,
}

impl IntoUtf32String for Data {
    fn into_utf32_string(self) -> crate::nucleo::Utf32String {
        self.ordinal.clone().into()
    }
}

impl PartialOrd for Data {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ordinal.partial_cmp(&other.ordinal) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.score.partial_cmp(&other.score)
    }
}

impl Scored for Data {
    fn score(&self) -> u32 {
        self.score as u32
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Default, PartialOrd, PartialEq)]
pub struct DataEntry {
    display: String,
}

pub struct Injector<T: Scored + IntoUtf32String + Clone>(crate::nucleo::Injector<T>);

impl<T: Scored + IntoUtf32String + Clone> From<crate::nucleo::Injector<T>> for Injector<T> {
    fn from(value: crate::nucleo::Injector<T>) -> Self {
        Self(value)
    }
}

impl<T: Scored + IntoUtf32String + Clone> Clone for Injector<T> {
    fn clone(&self) -> Self {
        <crate::nucleo::Injector<T> as Clone>::clone(&self.0).into()
    }
}

impl<T: Scored + IntoUtf32String + Clone> Injector<T> {
    pub fn push(&self, value: T) -> u32 {
        self.0
            .push(value.clone(), |dst| dst[0] = value.into_utf32_string())
    }
}

impl Injector<Data> {
    pub fn populate_with_source<C>(self, source: Source<C>) -> Result<()>
    where
        C: Partial
            + Debug
            + Clone
            + Sync
            + Send
            + Default
            + Serialize
            + for<'a> Deserialize<'a>
            + 'static,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Data>();

        let injector = source.clone();
        log::debug!("injector::populate_with_source");

        rt.block_on(async {
            let _f: JoinHandle<Result<()>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val.clone());
                }
                Ok(())
            });

            let finder = injector.finder_fn;

            finder(tx)
        })
    }

    pub fn populate_with_lua_source<C>(self, _lua: &Lua, _source: Source<C>) -> Result<()>
    where
        C: Partial
            + Debug
            + Clone
            + Sync
            + Send
            + Default
            + Serialize
            + for<'a> Deserialize<'a>
            + 'static,
    {
        todo!()
    }
}

pub struct Matcher<T: IntoUtf32String + Scored + Clone + Send + Sync + 'static>(
    crate::nucleo::Nucleo<T>,
);

impl<T: IntoUtf32String + Scored + Clone + Send + Sync + 'static> Matcher<T> {
    pub fn pattern(&mut self) -> &mut crate::nucleo::pattern::MultiPattern {
        &mut self.0.pattern
    }

    pub fn injector(&self) -> Injector<T> {
        self.0.injector().into()
    }

    pub fn tick(&mut self, timeout: u64) -> Status {
        self.0.tick(timeout)
    }

    pub fn snapshot(&self) -> &crate::nucleo::Snapshot<T> {
        self.0.snapshot()
    }

    pub fn restart(&mut self, clear_snapshot: bool) {
        self.0.restart(clear_snapshot)
    }
}

impl<T: IntoUtf32String + Scored + Clone + Send + Sync + 'static> From<Nucleo<T>> for Matcher<T> {
    fn from(value: Nucleo<T>) -> Self {
        Matcher(value)
    }
}

#[derive(Default)]
pub struct FuzzyMatcher(crate::nucleo::Matcher);

pub static MATCHER: Lazy<Arc<Mutex<FuzzyMatcher>>> =
    Lazy::new(|| Arc::new(Mutex::new(FuzzyMatcher::default())));

impl FuzzyMatcher {
    pub fn as_inner_mut(&mut self) -> &mut crate::nucleo::Matcher {
        &mut self.0
    }

    pub fn fuzzy_indices(
        &mut self,
        pattern: &Pattern,
        haystack: Utf32Str,
        indices: &mut Vec<u32>,
    ) -> Vec<(u32, u32)> {
        pattern.indices(haystack, self.as_inner_mut(), indices);

        indices.par_sort_unstable();
        indices.dedup();

        range_rover(indices.drain(..))
            .into_par_iter()
            .map(RangeInclusive::into_inner)
            .collect()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    #[default]
    Rust,
    Lua,
}

pub trait Finder {
    fn run(&self, tx: UnboundedSender<Data>) -> Result<()>;
    fn run_with_lua(&self, lua: &Lua, tx: UnboundedSender<Data>) -> Result<()>;
}

pub type FinderFn<T> = Arc<dyn Fn(UnboundedSender<T>) -> Result<()> + Sync + Send + 'static>;

#[derive(Clone, Builder)]
pub struct Source<C: Partial + Default + Clone> {
    origin: Origin,
    name: Sources,
    config: C,
    finder_fn: FinderFn<Data>,
}

impl<C: Partial + Default + Clone + Debug> Debug for Source<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("origin", &self.origin)
            .field("name", &self.name)
            .field("config", &self.config)
            // .field("finder_fn", &self.finder_fn)
            .finish()
    }
}

impl<C: Partial + Default + Clone> Source<C> {
    pub fn update_config(&mut self, config: impl Into<C>) {
        self.config = config.into();
    }
}

// impl<C: Clone + Default + Partial> Finder for Source<C> {
//     fn run(&self, tx: UnboundedSender<Data>) -> Result<()> {
//         todo!()
//     }
//
//     fn run_with_lua(&self, lua: &Lua, tx: UnboundedSender<Data>) -> Result<()> {
//         todo!()
//     }
// }

pub struct Picker<C>
where
    C: Partial
        + Default
        + Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static,
{
    pub matcher: Matcher<Data>,
    previous_query: String,
    cursor: Cursor,
    window: Window,
    selections: HashMap<String, Data>,
    sender: crossbeam_channel::Sender<()>,
    receiver: crossbeam_channel::Receiver<()>,
    config: Config,
    source: Source<C>,
}

#[buildstructor]
impl<
        C: Partial
            + Default
            + Clone
            + Debug
            + Sync
            + Send
            + Serialize
            + for<'a> Deserialize<'a>
            + 'static,
    > Picker<C>
{
    #[builder]
    pub fn new(source: Source<C>, config: Option<crate::config::Config>) -> Self {
        let config = config.unwrap_or_default();
        log::info!("Creating picker with config: {:?}", &config);
        let (sender, receiver) = bounded::<()>(1);
        // let notifier = sender.clone();
        // TODO: This hammers re-renders when loading lots of files. Is this even necessary?
        let notify = Arc::new(move || {
            // if let Err(err) = sender.try_send(()) {
            //     log::error!("Error sending notification: {:?}", err)
            // };
        });

        let matcher: Matcher<Data> = Nucleo::new(
            crate::nucleo::Config::DEFAULT.match_paths(),
            notify,
            None,
            1,
            false,
        )
        .into();

        Self {
            matcher,
            receiver,
            sender,
            config,
            source,
            cursor: Cursor::default(),
            previous_query: String::new(),
            selections: HashMap::new(),
            window: Window::new(50, 50),
        }
    }
}
impl<C> Picker<C>
where
    C: Partial
        + Default
        + Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static,
{
    pub fn tick(&mut self, timeout: u64) -> Status {
        let status = self.matcher.tick(timeout);
        if status.changed {
            self.force_rerender();
        }

        self.update_cursor();

        status
    }

    fn try_recv(&self) -> Result<()> {
        Ok(self.receiver.try_recv()?)
    }

    pub fn should_rerender(&self) -> bool {
        !self.receiver.is_empty()
    }

    pub fn force_rerender(&mut self) {
        let _ = self.sender.try_send(());
    }

    pub fn total_matches(&self) -> u32 {
        self.matcher.snapshot().matched_item_count()
    }

    pub fn total_items(&self) -> u32 {
        self.matcher.snapshot().item_count()
    }

    pub fn lower_bound(&self) -> u32 {
        max(self.window().start() as u32, 0).min(self.upper_bound())
    }

    pub fn upper_bound(&self) -> u32 {
        min(self.window.end() as u32, self.total_matches())
    }

    pub fn update_cursor(&mut self) {
        self.set_cursor_pos(self.cursor.pos());
    }

    pub fn window_width(&self) -> usize {
        self.window.width()
    }

    pub fn window_height(&self) -> usize {
        self.window.height()
    }

    pub fn update_window(&mut self, x: usize, y: usize) {
        self.set_window_width(x);
        self.set_window_height(y);
    }

    pub fn update_query(&mut self, query: &str) {
        let previous_query = self.previous_query.clone();
        if query != previous_query {
            log::debug!("Updating query: {}", &query);
            self.matcher.pattern().reparse(
                0,
                query,
                CaseMatching::Smart,
                Normalization::Smart,
                query.starts_with(&previous_query),
            );
            self.previous_query = query.to_string();
            // TODO: Debounce this tick? This whole function?
            // TODO: I feel like this can make this hitch scenarios where there's lots of matches...
            if self.config.selection_strategy().is_reset() {
                self.move_cursor_to(0);
            } else {
                self.tick(10);
            }
            self.force_rerender();
        }
    }

    pub fn update_config(&mut self, config: PartialConfig) {
        log::debug!("Updating config to: {:?}", config);
        self.config.apply_some(config);
    }

    pub fn move_cursor(&mut self, direction: Movement, change: u32) {
        log::debug!("Moving cursor {:?} by {}", direction, change);
        self.tick(10);

        if self.total_matches() == 0 {
            return;
        }

        let last_window_pos = self.window().start();

        let new_pos = match direction {
            Movement::Down => {
                self.cursor.pos().saturating_add(change as usize) as u32 % self.total_matches()
            }
            Movement::Up => {
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
    }

    pub fn move_cursor_to(&mut self, pos: usize) {
        log::debug!("Moving cursor to {}", pos);
        self.tick(10);

        if self.total_matches() == 0 {
            return;
        }

        let last_window_pos = self.window().start();
        self.set_cursor_pos(pos);
        if last_window_pos != self.window().start() {
            let _ = self.sender.try_send(());
        }
    }

    pub fn current_match_indices(&self, haystack: &str) -> Vec<(u32, u32)> {
        let mut match_indices = Vec::new();
        let snapshot = self.matcher.snapshot();
        let pattern = snapshot.pattern().column_pattern(0);
        let matcher = &mut MATCHER.lock();
        let mut buf = Vec::new();
        let indices = matcher.fuzzy_indices(
            pattern,
            Utf32Str::new(haystack, &mut buf),
            &mut match_indices,
        );

        indices
    }

    pub fn current_matches(&self) -> Vec<Data> {
        // let mut match_indices = Vec::new();
        let snapshot = self.matcher.snapshot();
        log::debug!("Item count: {:?}", snapshot.item_count());
        log::debug!("Match count: {:?}", snapshot.matched_item_count());
        // let matcher = &mut MATCHER.lock();

        let lower_bound = self.lower_bound();
        let upper_bound = self.upper_bound();

        snapshot
            .matched_items(lower_bound..upper_bound)
            .map(|item| {
                // let pattern = snapshot.pattern().column_pattern(0);
                // let indices = matcher.fuzzy_indices(
                //     pattern,
                //     item.matcher_columns[0].slice(..),
                //     &mut match_indices,
                // );

                let selected = self.selections.contains_key(&item.data.ordinal);
                if selected {
                    log::debug!("{:?} is selected", &item.data);
                }
                item.data.clone()
                // .with_indices(indices)
                // .with_selected(selected)
            })
            .collect::<Vec<_>>()
    }

    pub fn restart(&mut self) {
        self.matcher.restart(true);
    }

    // pub fn populate_with(&mut self, entries: Vec<Data>) -> Result<()> {
    //     let injector = self.matcher.injector();
    //     rayon::spawn(move || {
    //         injector.populate(entries);
    //     });
    //
    //     Ok(())
    // }

    pub fn multiselect(&mut self, index: u32) {
        let snapshot = self.matcher.snapshot();
        match snapshot.get_matched_item(index) {
            Some(entry) => {
                // WARN: This worries me...can these become out of sync?
                self.selections
                    .insert(entry.data.ordinal.clone(), entry.data.clone());
                log::debug!("multi-selections: {:?}", &self.selections);
            }
            None => {
                log::error!("Error multi-selecting index: {}", index);
            }
        };
    }

    pub fn deselect(&mut self, key: String) {
        self.selections.remove(&key);
    }

    pub fn toggle_selection(&mut self, index: u32) {
        let snapshot = self.matcher.snapshot();
        match snapshot.get_matched_item(index) {
            Some(entry) => {
                // WARN: This worries me...can these become out of sync?
                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.selections.entry(entry.data.ordinal.clone())
                {
                    e.insert(entry.data.clone());
                } else {
                    self.deselect(entry.data.ordinal.clone());
                }
                log::debug!("multi-selections: {:?}", &self.selections);
            }
            None => {
                log::error!("Error multi-selecting index: {}", index);
            }
        };
    }

    pub fn selections(&self) -> Vec<Data> {
        self.selections.clone().into_values().collect()
    }

    pub fn cursor_pos(&self) -> Option<u32> {
        if self.total_matches() == 0 {
            None
        } else {
            self.get_cursor_pos(Relative::Window).try_into().ok()
        }
    }

    pub fn shutdown(&mut self) {}
}

impl<C> Buffer<Data> for Picker<C>
where
    C: Partial
        + Default
        + Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static,
{
    fn len(&self) -> usize {
        self.total_matches().try_into().unwrap_or(usize::MAX)
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    fn cursor(&self) -> &crate::buffer::Cursor {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut crate::buffer::Cursor {
        &mut self.cursor
    }
}
impl<C> Picker<C>
where
    C: Partial
        + Default
        + Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static,
{
    pub fn populate(&self, lua: &Lua) -> Result<()> {
        let injector = self.matcher.injector();
        let source = self.source.clone();
        // if let Some(config) = config {
        //     source.update_config(config);
        // };

        match source.origin {
            Origin::Rust => {
                rayon::spawn(move || {
                    injector
                        .populate_with_source(source)
                        .expect("Failed populating!");
                });
            }
            Origin::Lua => {
                injector
                    .populate_with_lua_source(lua, source)
                    .expect("Failed populating!");
            }
        }

        Ok(())
    }
}

impl<C> UserData for Picker<C>
where
    C: Partial
        + Default
        + Clone
        + Debug
        + Sync
        + Send
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static,
{
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("update_query", |_lua, this, params: (String,)| {
            this.update_query(&params.0);
            Ok(())
        });

        methods.add_method_mut("update_config", |_lua, this, params: (PartialConfig,)| {
            this.update_config(params.0);
            Ok(())
        });

        methods.add_method("sort_direction", |_lua, this, ()| {
            log::debug!("{:?}", this.config.sort_direction());
            Ok(this.config.sort_direction())
        });

        methods.add_method_mut("move_cursor_up", |_lua, this, params: (Option<u32>,)| {
            let delta = params.0.unwrap_or(1);
            match this.config.sort_direction() {
                SortDirection::Descending => {
                    this.move_cursor(Movement::Up, delta);
                }
                SortDirection::Ascending => {
                    this.move_cursor(Movement::Down, delta);
                }
            }
            Ok(())
        });

        methods.add_method_mut("move_cursor_down", |_lua, this, params: (Option<u32>,)| {
            let delta = params.0.unwrap_or(1);
            match this.config.sort_direction() {
                SortDirection::Descending => {
                    this.move_cursor(Movement::Down, delta);
                }
                SortDirection::Ascending => {
                    this.move_cursor(Movement::Up, delta);
                }
            }
            Ok(())
        });

        methods.add_method_mut("move_to_top", |_lua, this, ()| {
            match this.config.sort_direction() {
                SortDirection::Descending => {
                    this.move_cursor_to(0);
                }
                SortDirection::Ascending => {
                    this.move_cursor_to(this.total_matches().saturating_sub(1) as usize);
                }
            }
            Ok(())
        });

        methods.add_method_mut("move_to_bottom", |_lua, this, ()| {
            match this.config.sort_direction() {
                SortDirection::Descending => {
                    this.move_cursor_to(this.total_matches().saturating_sub(1) as usize);
                }
                SortDirection::Ascending => {
                    this.move_cursor_to(0);
                }
            }
            Ok(())
        });

        methods.add_method_mut("set_cursor", |_lua, this, params: (usize,)| {
            this.set_cursor_pos_in_window(params.0);
            Ok(())
        });

        methods.add_method_mut("update_window", |_lua, this, params: (usize, usize)| {
            this.update_window(params.0, params.1);
            Ok(())
        });

        methods.add_method_mut("window_height", |_lua, this, ()| Ok(this.window_height()));

        methods.add_method("fuzzy_indices", |lua, this, params: (String,)| {
            let indices = this.current_match_indices(&params.0);

            Ok(lua.to_value(&indices))
        });

        methods.add_method("current_matches", |lua, this, ()| {
            Ok(lua.to_value(&this.current_matches()))
        });

        methods.add_method("total_items", |_lua, this, ()| Ok(this.total_items()));
        methods.add_method("total_matches", |_lua, this, ()| Ok(this.total_matches()));

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

        methods.add_method("selections", |lua, this, ()| {
            Ok(lua.to_value(&this.selections()))
        });

        methods.add_method_mut("multiselect", |_lua, this, params: (u32,)| {
            this.multiselect(params.0);
            Ok(())
        });

        methods.add_method_mut("toggle_selection", |_lua, this, params: (u32,)| {
            this.toggle_selection(params.0);
            Ok(())
        });

        methods.add_method_mut("tick", |lua, this, ms: u64| {
            let status = this.tick(ms);
            lua.to_value(&status)
        });

        methods.add_method_mut("populate", |lua, this, ()| {
            this.populate(lua).into_lua_err()
        });

        // methods.add_method_mut("populate_with", |lua, this, params: (LuaValue<'_>,)| {
        //     this.populate_with(lua.from_value(params.0)?).into_lua_err()
        // });

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
            let _ = this.try_recv();
            Ok(())
        });
    }
}
