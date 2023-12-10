use std::{collections::HashMap, sync::Arc};

use mlua::UserData;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::picker::{Entry, Picker};

#[derive(Default)]
pub struct State<T: Entry> {
    pickers: HashMap<String, Picker<T>>,
}

impl<T: Entry> State<T> {
    pub fn new() -> Self {
        Self { pickers: HashMap::new() }
    }
}

impl<T: Entry> UserData for State<T> {}
