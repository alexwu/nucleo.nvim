use mlua::UserData;
use nucleo::{Config, Nucleo};
use parking_lot::Mutex;
use std::ops::DerefMut;
use std::sync::Arc;

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

pub trait FuzzyPicker {
    // matcher: Nucleo<String>
    // previous_query: String
    fn matcher(&self) -> &Nucleo<String>;
    fn previous_query(&self) -> String;
}

pub struct Matcher(pub Nucleo<String>);

impl From<Nucleo<String>> for Matcher {
    fn from(value: Nucleo<String>) -> Self {
        Matcher(value)
    }
}

pub struct Picker {
    pub matcher: Matcher,
    previous_query: String,
}

impl Picker {
    pub fn new(cwd: String) -> Self {
        fn notify() {}
        let matcher: Matcher = Nucleo::new(Config::DEFAULT, Arc::new(notify), None, 1).into();
        // let injector = matcher.injector();
        // for item in items {
        //     injector.push(item.clone(), |dst| dst[0] = item.into());
        // }
        Self {
            matcher,
            previous_query: String::new(),
        }
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new(
            "".to_string(), // current_dir()
                            //     .expect("Couldn't get the current_dir")
                            //     .to_string_lossy()
                            //     .to_string(),
        )
    }
}

impl UserData for Picker {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {}
}

pub static PICKER: LazyMutex<Picker> = LazyMutex::new(Picker::default);
pub fn finder() -> impl DerefMut<Target = Picker> {
    PICKER.lock()
}
