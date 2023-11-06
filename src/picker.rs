use std::cmp::{max, min};
use std::path::Path;
use std::sync::{mpsc, Arc};

use ignore::types::TypesBuilder;
use ignore::{DirEntry, WalkBuilder, WalkState};
use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};
use nucleo::pattern::CaseMatching;
use nucleo::{Config, Nucleo};
use tokio::runtime::Runtime;

pub struct Matcher(pub Nucleo<String>);
pub struct Status(pub nucleo::Status);

impl Matcher {
    pub fn pattern(&mut self) -> &mut nucleo::pattern::MultiPattern {
        &mut self.0.pattern
    }

    pub fn injector(&mut self) -> nucleo::Injector<String> {
        self.0.injector()
    }

    pub fn tick(&mut self, timeout: u64) -> Status {
        Status(self.0.tick(timeout))
    }

    pub fn snapshot(&self) -> &nucleo::Snapshot<String> {
        self.0.snapshot()
    }
}

impl UserData for Status {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("changed", |_, this| Ok(this.0.changed));
        fields.add_field_method_get("running", |_, this| Ok(this.0.running));
    }
}

impl From<Nucleo<String>> for Matcher {
    fn from(value: Nucleo<String>) -> Self {
        Matcher(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
}

pub struct Picker {
    pub matcher: Matcher,
    previous_query: String,
    cwd: String,
    selection_index: u32,
    lower_bound: u32,
    upper_bound: u32,
}

impl Picker {
    pub fn new(cwd: String) -> Self {
        fn notify() {}
        let matcher: Matcher = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1).into();

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

    pub fn populate_picker(&mut self, git_ignore: bool) {
        log::info!("Populating picker with {}", &self.cwd);
        let runtime = Runtime::new().expect("Failed to create runtime");
        let cwd = self.cwd.clone();
        let injector = self.matcher.injector();

        let (tx, rx) = mpsc::channel::<String>();
        let _add_to_injector_thread = std::thread::spawn(move || -> anyhow::Result<()> {
            for val in rx.iter() {
                injector.push(val.clone(), |dst| dst[0] = val.into());
            }
            Ok(())
        });

        let _ = runtime.spawn(async move {
            let dir = Path::new(&cwd);
            log::info!("Spawning file searcher...");
            let mut walk_builder = WalkBuilder::new(dir.clone());
            walk_builder
                .hidden(true)
                .follow_links(true)
                .git_ignore(git_ignore)
                .sort_by_file_name(|name1, name2| name1.cmp(name2));
            let mut type_builder = TypesBuilder::new();
            type_builder
                .add(
                    "compressed",
                    "*.{zip,gz,bz2,zst,lzo,sz,tgz,tbz2,lz,lz4,lzma,lzo,z,Z,xz,7z,rar,cab}",
                )
                .expect("Invalid type definition");
            type_builder.negate("all");
            let excluded_types = type_builder
                .build()
                .expect("failed to build excluded_types");
            walk_builder.types(excluded_types);
            walk_builder.build_parallel().run(|| {
                let cwd = cwd.clone();
                let tx = tx.clone();
                Box::new(move |path: Result<DirEntry, ignore::Error>| -> WalkState {
                    match path {
                        Ok(file) if file.path().is_file() => {
                            let val = file
                                .path()
                                .strip_prefix(&cwd)
                                .expect("Failed to strip prefix")
                                .to_str()
                                .expect("Failed to convert path to string")
                                .to_string();
                            log::info!("Adding {}", &val);
                            // injector.push(val.clone(), |dst| dst[0] = val.into());
                            match tx.send(val.clone()) {
                                Ok(_) => WalkState::Continue,
                                Err(_) => WalkState::Skip,
                            }
                        }
                        Ok(_) => WalkState::Continue,
                        Err(_) => WalkState::Skip,
                    }
                })
            });
        });

        log::info!("After spawning file searcher...");
    }

    pub fn current_matches(&self) -> Vec<String> {
        let snapshot = self.matcher.snapshot();

        let lower_bound = self.lower_bound();
        let upper_bound = self.upper_bound();

        Vec::from_iter(
            snapshot
                .matched_items(lower_bound..upper_bound)
                .map(|item| item.data.clone()),
        )
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

impl UserData for Picker {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Call, |_, cwd: String| {
            let picker = Picker::new(cwd);
            Ok(picker)
        });

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

        methods.add_method("current_matches", |_lua, this, ()| {
            Ok(this.current_matches())
        });

        methods.add_method("get_selection_index", |_lua, this, ()| {
            Ok(this.selection_index)
        });

        methods.add_method("get_selection", |_lua, this, ()| {
            match this
                .matcher
                .snapshot()
                .get_matched_item(this.selection_index)
            {
                Some(selection) => Ok(selection.data.to_string()),
                None => Err(mlua::Error::runtime(std::format!(
                    "Failed getting the selection at selection_index: {}",
                    this.selection_index
                ))),
            }
        });

        methods.add_method_mut("tick", |_lua, this, ms: u64| Ok(this.matcher.tick(ms)));

        methods.add_method_mut("populate_picker", |_lua, this, _params: (String,)| {
            this.populate_picker(true);
            Ok(())
        });
    }
}
