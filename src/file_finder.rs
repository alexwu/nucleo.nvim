use crate::fuzzy::LazyMutex;
use ignore::WalkBuilder;
use ignore::{DirEntry, WalkState};
use mlua::UserData;
use nucleo::pattern::{Atom, AtomKind, CaseMatching};
use nucleo::{Config, Nucleo};
use parking_lot::Mutex;
use std::cmp::min;
use std::env::current_dir;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct FileFinder {
    pub matcher: Nucleo<String>,
    previous_query: String,
    pub cwd: String,
}

impl FileFinder {
    pub fn new(cwd: String) -> Self {
        fn notify() {}
        let matcher = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1);
        // let injector = matcher.injector();
        // for item in items {
        //     injector.push(item.clone(), |dst| dst[0] = item.into());
        // }
        Self {
            matcher,
            cwd,
            previous_query: String::new(),
        }
    }
}

impl Default for FileFinder {
    fn default() -> Self {
        Self::new(
            "".to_string(), // current_dir()
                            //     .expect("Couldn't get the current_dir")
                            //     .to_string_lossy()
                            //     .to_string(),
        )
    }
}

impl UserData for FileFinder {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods.add_async_method_mut("matches", |lua, this, size| async move {
        //     this.matcher.snapshot().matched_items(..10)
        // });
    }
}

pub static FINDER: LazyMutex<FileFinder> = LazyMutex::new(FileFinder::default);
pub fn finder() -> impl DerefMut<Target = FileFinder> {
    FINDER.lock()
}

pub fn insert_item(item: String) {
    // TODO: let's see what happens...picker().matcher.restart(false);
    let injector = finder().matcher.injector();
    injector.push(item.clone(), |dst| dst[0] = item.into());
}

fn update_query(query: &str) {
    let picker = &mut finder();
    let previous_query = picker.previous_query.clone();
    if query != previous_query {
        picker.matcher.pattern.reparse(
            0,
            query,
            CaseMatching::Smart,
            query.starts_with(&previous_query),
        );
        picker.previous_query = query.to_string();
        // finder().matcher.tick(10);
    }
}

pub fn update_cwd(cwd: &str) {
    let picker = &mut finder();
    picker.cwd = cwd.to_string();
}

pub async fn matches(query: &str) -> Vec<String> {
    update_query(query);

    let matcher = &mut finder().matcher;
    let status = matcher.tick(10);
    // status.running
    let snapshot = matcher.snapshot();
    // snapshot.clone_into

    let total_matches = snapshot.matched_item_count();
    let upper_bound = min(50, total_matches);

    Vec::from_iter(
        snapshot
            .matched_items(0..upper_bound)
            .map(|item| item.data.clone()),
    )
}

pub fn add_to_picker<'s>(
) -> Box<dyn FnMut(Result<DirEntry, ignore::Error>) -> WalkState + Send + 's> {
    let closure = |path: Result<DirEntry, ignore::Error>| match path {
        Ok(file) if file.path().is_file() => {
            let val = file
                .path()
                .strip_prefix(&finder().cwd)
                .expect("Failed to strip prefix")
                .to_str()
                .expect("Failed to convert path to string")
                .to_string();
            insert_item(val);
            let matcher = &mut finder().matcher;
            matcher.tick(10);
            WalkState::Continue
        }
        Ok(_) => WalkState::Continue,
        Err(_) => WalkState::Skip,
    };

    Box::new(closure)
}
pub async fn parallel_files(input: &str, git_ignore: bool) {
    let runtime = Runtime::new().unwrap();
    let dir_name = input.to_string();
    runtime.spawn(async move {
        let dir = Path::new(&dir_name);
        WalkBuilder::new(dir.clone())
            .hidden(true)
            .follow_links(true)
            .git_ignore(git_ignore)
            .build_parallel()
            .run(Box::new(add_to_picker));
    })
    .await;
}
