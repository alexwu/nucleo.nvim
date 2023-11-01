use nucleo::pattern::{Atom, AtomKind, CaseMatching};
use nucleo::Config;
use parking_lot::Mutex;
use std::ops::DerefMut;

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

pub static MATCHER: LazyMutex<nucleo::Matcher> = LazyMutex::new(nucleo::Matcher::default);

pub fn files(input: &str, git_ignore: bool) -> Vec<String> {
    use ignore::WalkBuilder;
    use std::path::Path;

    let dir = Path::new(input);
    WalkBuilder::new(&dir)
        .hidden(false)
        .follow_links(false) // We're scanning over depth 1
        .git_ignore(git_ignore)
        // .max_depth()
        .build()
        .filter_map(|file| {
            file.ok().and_then(|entry| {
                let is_file = entry.file_type().map_or(false, |entry| entry.is_file());

                if is_file {
                    let val = entry.path().to_str()?.to_string();
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
