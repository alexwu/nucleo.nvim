use std::sync::Arc;

use derive_deref::Deref;
use mlua::{UserData, UserDataFields};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rvstruct::ValueStruct;

use crate::entry::Scored;
use crate::nucleo::{self, Nucleo};
use crate::{entry::Entry, injector::Injector};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deref)]
pub struct Status(nucleo::Status);

impl UserData for Status {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("changed", |_, this| Ok(this.0.changed));
        fields.add_field_method_get("running", |_, this| Ok(this.0.running));
    }
}

pub struct Matcher<T: Entry + Scored + PartialOrd>(Nucleo<T>);

impl<T: Entry + Scored + PartialOrd> Matcher<T> {
    pub fn pattern(&mut self) -> &mut nucleo::pattern::MultiPattern {
        &mut self.0.pattern
    }

    pub fn injector(&self) -> Injector<T> {
        self.0.injector().into()
    }

    pub fn tick(&mut self, timeout: u64) -> Status {
        Status(self.0.tick(timeout))
    }

    pub fn snapshot(&self) -> &nucleo::Snapshot<T> {
        self.0.snapshot()
    }

    pub fn restart(&mut self, clear_snapshot: bool) {
        self.0.restart(clear_snapshot)
    }

    fn update_config(&mut self, config: nucleo::Config) {
        self.0.update_config(config);
    }
}

impl<T: Entry + Scored + PartialOrd> From<Nucleo<T>> for Matcher<T> {
    fn from(value: Nucleo<T>) -> Self {
        Matcher(value)
    }
}

#[derive(Default, ValueStruct)]
pub struct FuzzyMatcher(nucleo::Matcher);

pub static MATCHER: Lazy<Arc<Mutex<FuzzyMatcher>>> =
    Lazy::new(|| Arc::new(Mutex::new(FuzzyMatcher::default())));

impl FuzzyMatcher {
    pub fn as_inner_mut(&mut self) -> &mut nucleo::Matcher {
        &mut self.0
    }

    pub fn update_config(&mut self, config: nucleo::Config) {
        self.0.config = config;
    }
}
