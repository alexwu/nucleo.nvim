use nucleo::Nucleo;

use crate::{entry::Entry, injector::Injector};

pub struct Status(pub nucleo::Status);
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
