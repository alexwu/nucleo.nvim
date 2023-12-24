use std::{fmt::Debug, sync::Arc};

use anyhow::anyhow;
use partially::Partial;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{entry::Entry, injector::FinderFn, picker::Data, previewer::Previewable};

pub mod diagnostics;
pub mod files;
pub mod git;
pub mod lua_tables;

pub trait Populator<T, U, V>: Clone
where
    T: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    V: Entry,
{
    fn name(&self) -> String;
    fn update_config(&mut self, config: U);

    fn build_injector(&self) -> FinderFn<V>;
}

// pub trait Source<T, U>
// where
//     T: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
//     // U: Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
//     U: Previewable,
// {
//     fn name(&self) -> String;
//     fn finder(&self) -> InjectorFn<T, U, U>;
//     fn config(&self) -> U;
// }

// pub type OtherFinderFn<T, U> = Arc<dyn Fn(UnboundedSender<Data<T, U>>) + Sync + Send + 'static>;
pub type InjectorBuilderFn<T, U> = Arc<dyn Fn(U) -> FinderFn<T> + Sync + Send>;

#[derive(Clone)]
pub struct Source<T, U>
where
    T: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Previewable + Partial,
{
    name: String,
    config: U,
    injector: InjectorBuilderFn<T, U>,
    // injector: FinderFn<T, U>,
}

#[buildstructor::buildstructor]
impl<
        T: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
        U: Previewable + Partial,
    > Source<T, U>
{
    #[builder]
    pub fn new<S: Into<String>>(name: S, injector: InjectorBuilderFn<T, U>, config: U) -> Self {
        Self {
            name: name.into(),
            injector,
            config,
        }
    }

    pub fn build_injector(&self) -> FinderFn<T> {
        // TODO: Make this so it merges the two
        // let config = match opts {
        //     Some(opts) => opts,
        //     None => self.config.clone(),
        // };

        let builder = self.injector.clone();

        builder(self.config.clone())
    }
}

impl<T: Debug, U: Debug> Debug for Source<T, U>
where
    T: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
    U: Previewable + Partial,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("name", &self.name)
            // .field("finder", &self.finder)
            .field("config", &self.config)
            .finish()
    }
}
