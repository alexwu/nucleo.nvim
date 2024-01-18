use std::{env::current_dir, marker::PhantomData, path::PathBuf, sync::Arc};

use buildstructor::Builder;
use enum_dispatch::enum_dispatch;
use mlua::Lua;
use partially::Partial;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::custom::Custom;
use super::Sources;
use crate::entry::{Entry, IntoData};
use crate::error::Result;
use crate::files::FileFinder;
use crate::previewer::PreviewOptions;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    File,
    Diff,
    Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Default)]
pub enum Location {
    Buffer(usize),
    File(PathBuf),
    #[default]
    None,
}

impl From<PathBuf> for Location {
    fn from(value: PathBuf) -> Self {
        Self::File(value)
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, Default)]
pub struct SimpleData {
    kind: Kind,
    ordinal: String,
    location: Location,
    score: usize,
    preview_options: PreviewOptions,
}

#[enum_dispatch]
pub trait Finder {
    fn run(&self, lua: &Lua, tx: UnboundedSender<SimpleData>) -> Result<()>;
}

#[enum_dispatch(Finder)]
#[derive(Clone, Debug)]
pub enum FinderFunction<T: Entry + Into<SimpleData>> {
    FileFinder(FileFinder),
    Custom(Custom<T>),
}

#[derive(Debug, Clone, Builder)]
pub struct Source<T: Entry + Into<SimpleData>, C> {
    name: Sources,
    config: C,
    finder_fn: FinderFunction<T>,
}

// impl<'a, T, C, F> Deserialize<'a> for Source<C, F>
// where
//     C: Deserialize<'a>,
//     F: Deserialize<'a>,
// {
//     fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'a>,
//     {
//         todo!()
//     }
// }
