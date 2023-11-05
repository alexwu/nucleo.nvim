use ignore::{DirEntry, WalkBuilder, WalkState};
use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};
use nucleo::pattern::CaseMatching;
use nucleo::{Config, Nucleo};
use parking_lot::Mutex;
use std::cmp::min;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct LazyMutex<T> {
    inner: Mutex<Option<T>>,
    init: fn() -> T,
}

impl<T> LazyMutex<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(None),
            init,
        }
    }

    pub fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        parking_lot::MutexGuard::map(self.inner.lock(), |val| val.get_or_insert_with(self.init))
    }
}

#[derive(Debug, Clone)]
pub enum EntryKind {
    File,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub kind: EntryKind,
    pub value: String,
    pub score: f32,
}

pub struct Matcher(pub Nucleo<String>);

impl Matcher {
    pub fn pattern(&mut self) -> &mut nucleo::pattern::MultiPattern {
        &mut self.0.pattern
    }
    pub fn injector(&mut self) -> nucleo::Injector<String> {
        self.0.injector()
    }
    pub fn tick(&mut self, timeout: u64) -> nucleo::Status {
        self.0.tick(timeout)
    }

    pub fn snapshot(&self) -> &nucleo::Snapshot<String> {
        self.0.snapshot()
    }
}

impl From<Nucleo<String>> for Matcher {
    fn from(value: Nucleo<String>) -> Self {
        Matcher(value)
    }
}

pub struct Picker {
    pub matcher: Matcher,
    previous_query: String,
    cwd: String,
}

impl Picker {
    pub fn new(cwd: String) -> Self {
        fn notify() {}
        let matcher: Matcher = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1).into();
        Self {
            matcher,
            cwd,
            previous_query: String::new(),
        }
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

    pub fn populate_picker(&mut self, input: &str, git_ignore: bool) {
        let runtime = Runtime::new().expect("Failed to create runtime");
        let dir_name = input.to_string();
        let cwd = self.cwd.clone();
        let injector = self.matcher.injector();

        runtime.spawn(async move {
            let dir = Path::new(&dir_name);
            log::info!("Spawning file searcher...");
            WalkBuilder::new(dir.clone())
                .hidden(true)
                .follow_links(true)
                .git_ignore(git_ignore)
                .sort_by_file_name(|name1, name2| name1.cmp(name2))
                .build_parallel()
                .run(|| {
                    Box::new(|path: Result<DirEntry, ignore::Error>| -> WalkState {
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
                                injector.push(val.clone(), |dst| dst[0] = val.into());
                                WalkState::Continue
                            }
                            Ok(_) => WalkState::Continue,
                            Err(_) => WalkState::Skip,
                        }
                    })
                });
        });
    }

    pub fn current_matches(&self) -> Vec<String> {
        let snapshot = self.matcher.snapshot();

        let total_matches = snapshot.matched_item_count();
        let upper_bound = min(50, total_matches);

        Vec::from_iter(
            snapshot
                .matched_items(0..upper_bound)
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
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Call, |_, cwd: String| {
            let picker = Picker::new(cwd);
            Ok(picker)
        });

        methods.add_method_mut("update_query", |lua, this, params: (String,)| {
            this.update_query(params.0);
            this.matcher.tick(10);
            Ok(())
        });

        methods.add_method(
            "current_matches",
            |lua, this, ()| Ok(this.current_matches()),
        );
    }
}
