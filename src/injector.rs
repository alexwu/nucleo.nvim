use std::fmt::Debug;
use std::sync::Arc;

use mlua::{FromLua, Lua};
use partially::Partial;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{runtime::Runtime, sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    entry::{IntoUtf32String, Scored},
    error::Result,
    sources::Populator,
};

pub type FinderFn<T> = Arc<dyn Fn(UnboundedSender<T>) -> Result<()> + Sync + Send + 'static>;

pub struct Injector<T: IntoUtf32String + Scored>(crate::nucleo::Injector<T>);

impl<T: IntoUtf32String + Scored> From<crate::nucleo::Injector<T>> for Injector<T> {
    fn from(value: crate::nucleo::Injector<T>) -> Self {
        Self(value)
    }
}

impl<T: IntoUtf32String + Scored> Clone for Injector<T> {
    fn clone(&self) -> Self {
        <crate::nucleo::Injector<T> as Clone>::clone(&self.0).into()
    }
}

impl<T: IntoUtf32String + Scored> Injector<T> {
    pub fn push(&self, value: T) -> u32 {
        let val = value.into_utf32_string();
        self.0.push(value, |dst| dst[0] = val)
    }
}

impl<T: IntoUtf32String + Send + Scored + 'static> Injector<T> {
    pub fn populate(self, entries: Vec<T>) {
        log::debug!("Populating picker with {} entries", entries.len());
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        let sender = tx.clone();
        rt.block_on(async {
            let _add_to_injector_thread: JoinHandle<Result<()>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val);
                }
                Ok(())
            });

            entries.into_par_iter().for_each(|entry| {
                let _ = sender.send(entry);
            });
        });
    }

    pub fn populate_with_source<P, U, V>(self, mut source: P) -> Result<()>
    where
        U: Debug + Sync + Send + Default + Serialize + for<'a> Deserialize<'a> + 'static,
        V: Debug + Sync + Send + Serialize + for<'a> Deserialize<'a> + 'static,
        P: Populator<V, U, T>,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        let injector = source.build_injector(None);
        log::debug!("injector::populate_with_source");
        rt.block_on(async {
            let _f: JoinHandle<Result<()>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val);
                }
                Ok(())
            });

            injector(tx)
        })
    }

    pub fn populate_with_lua_source<P, U, V>(self, lua: &Lua, mut source: P) -> Result<()>
    where
        U: Debug + Sync + Send + Serialize + Default + for<'a> Deserialize<'a>,
        V: Debug + Sync + Send + Serialize + for<'a> Deserialize<'a>,
        P: Populator<V, U, T>,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

        log::debug!("Before build_injector");
        let injector = source.build_injector(Some(lua));
        log::debug!("injector::populate_with_lua_source");

        rt.block_on(async {
            let _f: JoinHandle<Result<()>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val);
                }
                Ok(())
            });

            injector(tx)
        })
    }
}

pub trait Config:
    Serialize + Debug + Default + for<'a> Deserialize<'a> + FromLua + Sync + Send
{
}

impl<T> Config for T where
    T: Serialize + Debug + Default + for<'a> Deserialize<'a> + FromLua + Sync + Send + Clone
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
