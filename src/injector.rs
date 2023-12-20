use std::sync::Arc;

use crossbeam_channel::{unbounded, Sender};
use rayon::prelude::*;
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::entry::Entry;

pub struct Injector<T: Entry>(nucleo::Injector<T>);

impl<T: Entry> From<nucleo::Injector<T>> for Injector<T> {
    fn from(value: nucleo::Injector<T>) -> Self {
        Self(value)
    }
}

impl<T: Entry> Clone for Injector<T> {
    fn clone(&self) -> Self {
        <nucleo::Injector<T> as Clone>::clone(&self.0).into()
    }
}

impl<T: Entry> Injector<T> {
    pub fn push(&self, value: T) -> u32 {
        self.0
            .push(value.clone(), |dst| dst[0] = value.ordinal().into())
    }

    pub fn populate(self, entries: Vec<T>) {
        log::info!("Populating picker with {} entries", entries.len());
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let sender = tx.clone();
        rt.block_on(async {
            let _add_to_injector_thread: JoinHandle<Result<(), _>> = rt.spawn(async move {
                while let Some(val) = rx.recv().await {
                    self.push(val);
                }
                anyhow::Ok(())
            });

            entries.into_par_iter().for_each(|entry| {
                let _ = sender.send(entry);
            });
        });
    }

    pub fn populate_with<F>(self, func: Arc<F>)
    where
        F: Fn(Sender<T>) + Sync + Send + ?Sized + 'static,
    {
        let rt = Runtime::new().expect("Failed to create runtime");

        let (tx, rx) = unbounded::<T>();

        log::info!("injector::populate_with");
        rt.block_on(async {
            let _add_to_injector_thread: JoinHandle<Result<(), _>> = rt.spawn(async move {
                for val in rx.iter() {
                    self.push(val);
                }
                anyhow::Ok(())
            });

            func(tx);
        });
    }

    pub fn populate_with_local<F>(self, func: F)
    where
        F: Fn(Sender<T>) + 'static,
    {
        let runtime = Runtime::new().expect("Failed to create runtime");
        let (tx, rx) = unbounded::<T>();
        let _add_to_injector_thread: JoinHandle<Result<(), _>> = runtime.spawn(async move {
            for val in rx.iter() {
                log::info!("Sending local: {:?}", &val);
                self.push(val);
            }
            anyhow::Ok(())
        });

        log::info!("injector::populate_with_local");
        func(tx);
    }
}
