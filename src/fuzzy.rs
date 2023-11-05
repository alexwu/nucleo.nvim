use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;
use nucleo::pattern::{Atom, AtomKind, CaseMatching};
use nucleo::{Config, Nucleo};
use parking_lot::Mutex;

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

pub struct Picker {
    pub matcher: Nucleo<String>,
    previous_query: String,
}

impl Picker {
    pub fn new(
        items: Vec<String>,
        starting_query: Option<String>,
        // callback_fn: impl Fn(),
    ) -> Self {
        fn notify() {}
        let matcher = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1);
        let injector = matcher.injector();
        for item in items {
            injector.push(item.clone(), |dst| dst[0] = item.into());
        }
        Self {
            matcher,
            previous_query: starting_query.unwrap_or_default(),
        }
    }
}

impl Default for Picker {
    fn default() -> Self {
        Picker::new(vec![], None)
    }
}

pub static PICKER: LazyMutex<Picker> = LazyMutex::new(Picker::default);
pub static MATCHER: LazyMutex<nucleo::Matcher> = LazyMutex::new(nucleo::Matcher::default);

pub fn picker() -> impl DerefMut<Target = Picker> {
    PICKER.lock()
}

pub fn restart_picker() {
    picker().matcher.restart(true)
}

pub fn set_picker_items(items: Vec<String>) {
    // TODO: let's see what happens...picker().matcher.restart(false);
    let injector = picker().matcher.injector();
    for item in items {
        injector.push(item.clone(), |dst| dst[0] = item.into());
    }
}

pub fn update_query(query: &str) {
    let picker = &mut picker();
    let previous_query = picker.previous_query.clone();
    if query != previous_query {
        picker.matcher.pattern.reparse(
            0,
            query,
            CaseMatching::Smart,
            query.starts_with(&previous_query),
        );
        picker.previous_query = query.to_string();
    }
}

pub fn matches() -> Vec<String> {
    let matcher = &mut picker().matcher;
    matcher.tick(10);
    let snapshot = matcher.snapshot();

    let total_matches = snapshot.matched_item_count();

    Vec::from_iter(
        snapshot
            .matched_items(0..total_matches)
            .map(|item| item.data.clone()),
    )
}

pub fn files(input: &str, git_ignore: bool) -> Vec<String> {
    let dir = Path::new(input);
    WalkBuilder::new(dir)
        .hidden(true)
        .follow_links(true)
        .git_ignore(git_ignore)
        .build()
        .filter_map(|file| {
            file.ok().and_then(|entry| {
                let is_file = entry.file_type().map_or(false, |entry| entry.is_file());

                if is_file {
                    let val = entry.path().strip_prefix(dir).ok()?.to_str()?.to_string();
                    Some(val)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// NOTE: From Helix:
/// convenience function to easily fuzzy match
/// on a (relatively small list of inputs). This is not recommended for building a full tui
/// application that can match large numbers of matches as all matching is done on the current
/// thread, effectively blocking the UI
pub fn fuzzy_match<T: AsRef<str>>(
    pattern: &str,
    items: impl IntoIterator<Item = T>,
    path: bool,
) -> Vec<(T, u16)> {
    let mut matcher = MATCHER.lock();
    matcher.config = Config::DEFAULT;
    if path {
        matcher.config.set_match_paths();
    }
    let pattern = Atom::new(pattern, CaseMatching::Smart, AtomKind::Fuzzy, false);
    pattern.match_list(items, &mut matcher)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_test() {
        let query = "test";
        let items = files("/Users/jamesbombeelu/Code/", true);
        let result = fuzzy_match(query, items, false);

        assert_eq!(result.len(), 100)
    }
}
