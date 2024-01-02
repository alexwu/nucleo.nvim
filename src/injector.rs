use std::fmt::Debug;
use std::sync::Arc;

use mlua::{FromLua, Lua};
use partially::Partial;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{runtime::Runtime, sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    entry::{IntoUtf32String, Scored},
    sources::Populator,
};

pub type FinderFn<T> =
    Arc<dyn Fn(UnboundedSender<T>) -> anyhow::Result<()> + Sync + Send + 'static>;

pub struct Injector<T: IntoUtf32String + Scored + Clone>(crate::nucleo::Injector<T>);

impl<T: IntoUtf32String + Scored + Clone> From<crate::nucleo::Injector<T>> for Injector<T> {
    fn from(value: crate::nucleo::Injector<T>) -> Self {
        Self(value)
    }
}

impl<T: IntoUtf32String + Scored + Clone> Clone for Injector<T> {
    fn clone(&self) -> Self {
        <crate::nucleo::Injector<T> as Clone>::clone(&self.0).into()
    }
}

impl<T: IntoUtf32String + Clone + Scored> Injector<T> {
    pub fn push(&self, value: T) -> u32 {
        self.0
            .push(value.clone(), |dst| dst[0] = value.into_utf32_string())
    }
}

impl<T: IntoUtf32String + Clone + Send + Scored + 'static> Injector<T> {
    pub fn populate(self, entries: Vec<T>) {
        log::info!("Populating picker with {} entries", entries.len());
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        let sender = tx.clone();
        rt.block_on(async {
            let _add_to_injector_thread: JoinHandle<Result<(), _>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val.clone());
                }
                anyhow::Ok(())
            });

            entries.into_par_iter().for_each(|entry| {
                let _ = sender.send(entry);
            });
        });
    }

    pub fn populate_with_source<P, U, V>(self, source: P) -> anyhow::Result<()>
    where
        U: Debug + Clone + Sync + Send + Default + Serialize + for<'a> Deserialize<'a> + 'static,
        V: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
        P: Populator<V, U, T>,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        let injector = source.build_injector(None);
        log::info!("injector::populate_with_source");
        rt.block_on(async {
            let _f: JoinHandle<Result<(), _>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val.clone());
                }
                anyhow::Ok(())
            });

            injector(tx)
        })
    }

    pub fn populate_with_lua_source<P, U, V>(self, lua: &Lua, source: P) -> anyhow::Result<()>
    where
        U: Debug + Clone + Sync + Send + Serialize + Default + for<'a> Deserialize<'a> + 'static,
        V: Debug + Clone + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
        P: Populator<V, U, T>,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        log::info!("Before build_injector");
        let injector = source.build_injector(Some(lua));
        log::info!("injector::populate_with_lua_source");

        rt.block_on(async {
            let _f: JoinHandle<Result<(), _>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val.clone());
                }
                anyhow::Ok(())
            });

            injector(tx)
        })
    }
}

pub trait Config:
    Serialize + Debug + Clone + Default + for<'a> Deserialize<'a> + for<'a> FromLua<'a> + Sync + Send
{
}

impl<T> Config for T where
    T: Serialize
        + Debug
        + Clone
        + Default
        + for<'a> Deserialize<'a>
        + for<'a> FromLua<'a>
        + Sync
        + Send
{
}

pub trait FromPartial: Default + Partial {
    fn from_partial(value: Self::Item) -> Self {
        let mut config = Self::default();
        config.apply_some(value);
        config
    }
}

impl<T> FromPartial for T
where
    T: Partial + Default,
    T::Item: for<'a> Deserialize<'a>,
{
}
