use mlua::Lua;
use tokio::sync::mpsc::UnboundedSender;

use super::source::{Finder, SimpleData};
use crate::{
    error::Result,
};

#[derive(Clone, Debug)]
pub struct Custom<T: Into<SimpleData>> {
    results: Vec<T>,
}

impl<T: Into<SimpleData>> Finder for Custom<T> {
    fn run(&self, _lua: &Lua, _tx: UnboundedSender<SimpleData>) -> Result<()> {
        todo!()
    }
}
