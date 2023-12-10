use std::collections::HashMap;

use mlua::UserData;

use crate::picker::{Entry, Picker};

#[derive(Default)]
pub struct State<T: Entry> {
    pickers: HashMap<String, Picker<T>>,
}

impl<T: Entry> State<T> {
    pub fn new() -> Self {
        Self {
            pickers: HashMap::new(),
        }
    }
}

impl<T: Entry> UserData for State<T> {}
