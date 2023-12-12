use std::sync::Arc;

use mlua::{UserData, UserDataFields};
use nucleo::Nucleo;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::{entry::Entry, injector::Injector};

pub struct Status(pub nucleo::Status);
impl UserData for Status {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("changed", |_, this| Ok(this.0.changed));
        fields.add_field_method_get("running", |_, this| Ok(this.0.running));
    }
}

pub struct Matcher<T: Entry>(pub Nucleo<T>);

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

impl<T: Entry> From<Nucleo<T>> for Matcher<T> {
    fn from(value: Nucleo<T>) -> Self {
        Matcher(value)
    }
}

impl<T: Entry> Matcher<T> {
    fn update_config(&mut self, config: nucleo::Config) {
        self.0.update_config(config);
    }
}

#[derive(Default)]
pub struct StringMatcher(pub nucleo::Matcher);

pub static STRING_MATCHER: Lazy<Arc<Mutex<StringMatcher>>> =
    Lazy::new(|| Arc::new(Mutex::new(StringMatcher::default())));

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

impl StringMatcher {
    pub fn as_inner_mut(&mut self) -> &mut nucleo::Matcher {
        &mut self.0
    }

    pub fn update_config(&mut self, config: nucleo::Config) {
        self.0.config = config;
    }
}
