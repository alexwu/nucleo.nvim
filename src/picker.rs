use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Debug;
use std::string::ToString;
use std::sync::Arc;

use buildstructor::buildstructor;
use crossbeam_channel::bounded;
use mlua::ExternalResult;
use mlua::{
    prelude::{Lua, LuaResult, LuaValue},
    FromLua, LuaSerdeExt, UserData, UserDataMethods,
};
use nucleo_matcher::Utf32Str;
use partially::Partial;
use serde::{Deserialize, Serialize};

use crate::buffer::{Buffer, Cursor, Relative};
use crate::config::{Config, PartialConfig, SortDirection};
use crate::entry::{Data, Entry};
use crate::injector::{Config as InjectorConfig, FromPartial};
use crate::matcher::{Matcher, Status, MATCHER};
use crate::nucleo::pattern::{CaseMatching, Normalization};
use crate::nucleo::Nucleo;
use crate::sources::{Populator, SourceKind};
use crate::window::Window;

#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::Display, Default, Partial)]
#[partially(derive(Default, Debug, Clone, Serialize, Deserialize, FromLua))]
pub struct Blob {
    pub inner: serde_json::Value,
}
impl<'a> FromLua<'a> for Blob {
    fn from_lua(value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        let ty = value.type_name();
        Ok(Blob {
            inner: serde_json::to_value(value).map_err(|e| {
                mlua::Error::FromLuaConversionError {
                    from: ty,
                    to: "Blob",
                    message: Some(format!("{}", e)),
                }
            })?,
        })
    }
}

pub struct Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig + 'static,
    P: Populator<T, V, Data<T>>,
{
    pub matcher: Matcher<Data<T>>,
    previous_query: String,
    cursor: Cursor,
    window: Window,
    selections: HashMap<String, Data<T>>,
    sender: crossbeam_channel::Sender<()>,
    receiver: crossbeam_channel::Receiver<()>,
    config: Config,
    source: P,
    _marker: std::marker::PhantomData<V>,
}

#[buildstructor]
impl<T, V, P> Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig + 'static,
    P: Populator<T, V, Data<T>> + Clone,
{
    #[builder]
    pub fn new(source: P, config: Option<crate::config::Config>, multi_sort: Option<bool>) -> Self {
        let config = config.unwrap_or_default();
        log::info!("Creating picker with config: {:?}", &config);
        let (sender, receiver) = bounded::<()>(1);
        // let notifier = sender.clone();
        // TODO: This hammers re-renders when loading lots of files. Is this even necessary?
        let notify = Arc::new(move || {
            // if notifier.try_send(()).is_ok() {
            //     log::info!("Message sent!")
            // };
        });

        let matcher: Matcher<Data<T>> = Nucleo::new(
            crate::nucleo::Config::DEFAULT.match_paths(),
            notify,
            None,
            1,
            multi_sort.unwrap_or_default(),
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
            _marker: Default::default(),
        }
    }
}
impl<T, V, P> Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig,
    P: Populator<T, V, Data<T>> + Clone + Send + 'static,
{
    pub fn tick(&mut self, timeout: u64) -> Status {
        let status = self.matcher.tick(timeout);
        if status.changed {
            self.force_rerender();
        }

        self.update_cursor();

        status
    }

    fn try_recv(&self) -> Result<(), crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
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
            log::info!("Updating query: {}", &query);
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
        // log::info!("Updating config to: {:?}", config);
        self.config.apply_some(config);
    }

    pub fn move_cursor(&mut self, direction: Movement, change: u32) {
        log::info!("Moving cursor {:?} by {}", direction, change);
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
        log::info!("Moving cursor to {}", pos);
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

    pub fn current_matches(&self) -> Vec<Data<T>> {
        let mut match_indices = Vec::new();
        let snapshot = self.matcher.snapshot();
        log::info!("Item count: {:?}", snapshot.item_count());
        log::info!("Match count: {:?}", snapshot.matched_item_count());
        let matcher = &mut MATCHER.lock();

        let lower_bound = self.lower_bound();
        let upper_bound = self.upper_bound();

        snapshot
            .matched_items(lower_bound..upper_bound)
            .map(|item| {
                let pattern = snapshot.pattern().column_pattern(0);
                let indices = matcher.fuzzy_indices(
                    pattern,
                    item.matcher_columns[0].slice(..),
                    &mut match_indices,
                );

                let selected = self.selections.contains_key(&item.data.ordinal());
                if selected {
                    log::info!("{:?} is selected", &item.data);
                }
                item.data
                    .clone()
                    .with_indices(indices)
                    .with_selected(selected)
            })
            .collect::<Vec<_>>()
    }

    pub fn restart(&mut self) {
        self.matcher.restart(true);
    }

    pub fn populate_with(&mut self, entries: Vec<Data<T>>) -> anyhow::Result<()> {
        let injector = self.matcher.injector();
        rayon::spawn(move || {
            injector.populate(entries);
        });

        Ok(())
    }

    pub fn multiselect(&mut self, index: u32) {
        let snapshot = self.matcher.snapshot();
        match snapshot.get_matched_item(index) {
            Some(entry) => {
                // WARN: This worries me...can these become out of sync?
                self.selections
                    .insert(entry.data.ordinal(), entry.data.clone());
                log::info!("multi-selections: {:?}", &self.selections);
            }
            None => {
                log::info!("Error multi-selecting index: {}", index);
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
                    self.selections.entry(entry.data.ordinal())
                {
                    e.insert(entry.data.clone());
                } else {
                    self.deselect(entry.data.ordinal());
                }
                log::info!("multi-selections: {:?}", &self.selections);
            }
            None => {
                log::info!("Error multi-selecting index: {}", index);
            }
        };
    }

    pub fn selections(&self) -> Vec<Data<T>> {
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

impl<T, V, P> Buffer<T> for Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig,
    P: Populator<T, V, Data<T>> + Clone + Send + 'static,
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
impl<T, V, P> Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig + FromPartial,
    V::Item: Debug,
    P: Populator<T, V, Data<T>> + Send + Clone + 'static,
{
    pub fn populate(&self, lua: &Lua, config: Option<V::Item>) -> anyhow::Result<()> {
        let injector = self.matcher.injector();
        let mut source = self.source.clone();
        if let Some(config) = config {
            source.update_config(V::from_partial(config));
        };

        match source.kind() {
            SourceKind::Rust => {
                rayon::spawn(move || {
                    injector
                        .populate_with_source(source)
                        .expect("Failed populating!");
                });
            }
            SourceKind::Lua => {
                injector
                    .populate_with_lua_source(lua, source)
                    .expect("Failed populating!");
            }
        }

        Ok(())
    }
}

impl<T, V, P> UserData for Picker<T, V, P>
where
    T: Clone + Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: InjectorConfig + FromPartial,
    V::Item: Debug,
    P: Populator<T, V, Data<T>> + Clone + Send + 'static,
    (std::option::Option<<V as partially::Partial>::Item>,): for<'lua> mlua::FromLuaMulti<'lua>,
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

            log::info!("indices? {:?}", &indices);

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

        methods.add_method_mut("tick", |_lua, this, ms: u64| {
            let status = this.tick(ms);
            Ok(status)
        });

        methods.add_method_mut("populate", |lua, this, params: (Option<V::Item>,)| {
            this.populate(lua, params.0).into_lua_err()
        });

        methods.add_method_mut("populate_with", |lua, this, params: (LuaValue<'_>,)| {
            this.populate_with(lua.from_value(params.0)?).into_lua_err()
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
            let _ = this.try_recv();
            Ok(())
        });
    }
}
