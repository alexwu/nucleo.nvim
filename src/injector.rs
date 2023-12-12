use std::{path::Path, sync::Arc};

use crossbeam_channel::{unbounded, Sender};
use ignore::{types::TypesBuilder, WalkBuilder};
use rayon::prelude::*;
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::{entry::Entry, sources::files::FileConfig};

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
            .push(value.clone(), |dst| dst[0] = value.display().into())
    }

    pub fn populate(self, entries: Vec<T>) {
        log::info!("Populating picker with {} entries", entries.len());
        let runtime = Runtime::new().expect("Failed to create runtime");

        let (tx, rx) = unbounded::<T>();
        let _add_to_injector_thread: JoinHandle<Result<(), _>> = runtime.spawn(async move {
            for val in rx.iter() {
                self.push(val);
            }
            anyhow::Ok(())
        });
        runtime.spawn(async move {
            entries.into_par_iter().for_each(|entry| {
                let _ = tx.send(entry);
            });
        });
    }

    pub fn populate_with<F>(self, func: Arc<F>)
    where
        F: Fn(Sender<T>) + Sync + Send + ?Sized + 'static,
    {
        let runtime = Runtime::new().expect("Failed to create runtime");

        let (tx, rx) = unbounded::<T>();
        let _add_to_injector_thread: JoinHandle<Result<(), _>> = runtime.spawn(async move {
            for val in rx.iter() {
                self.push(val);
            }
            anyhow::Ok(())
        });

        log::info!("injector::populate_with");
        rayon::spawn(move || {
            func(tx)
        });
        // runtime.spawn(async move {
        //     log::info!("nside closure");
        //     func(tx);
        // });
    }

    pub fn populate_files_sorted(self, cwd: String, git_ignore: bool, ignore: bool, hidden: bool) {
        log::info!("Populating picker with {}", &cwd);
        let runtime = Runtime::new().expect("Failed to create runtime");

        let (tx, rx) = unbounded::<T>();
        let _add_to_injector_thread: JoinHandle<Result<(), _>> = runtime.spawn(async move {
            for val in rx.iter() {
                self.push(val);
            }
            anyhow::Ok(())
        });

        let config = FileConfig {
            cwd,
            git_ignore,
            ignore,
            hidden,
        };

        runtime.spawn(async move {
            // injection_fn(tx.clone());
            // let dir = Path::new(&cwd);
            // log::info!("Spawning sorted file searcher...");
            // let mut walk_builder = WalkBuilder::new(dir);
            // walk_builder
            //     .hidden(hidden)
            //     .follow_links(true)
            //     .git_ignore(git_ignore)
            //     .ignore(ignore)
            //     .sort_by_file_name(std::cmp::Ord::cmp);
            //
            // let mut type_builder = TypesBuilder::new();
            // type_builder
            //     .add(
            //         "compressed",
            //         "*.{zip,gz,bz2,zst,lzo,sz,tgz,tbz2,lz,lz4,lzma,lzo,z,Z,xz,7z,rar,cab}",
            //     )
            //     .expect("Invalid type definition");
            // type_builder.negate("all");
            // let excluded_types = type_builder
            //     .build()
            //     .expect("failed to build excluded_types");
            // walk_builder.types(excluded_types);
            // let tx = tx.clone();
            // for path in walk_builder.build() {
            //     let cwd = cwd.clone();
            //     match path {
            //         Ok(file) if file.path().is_file() => {
            //             if tx
            //                 .send(Entry::from_path(file.path(), Some(cwd.clone())))
            //                 .is_ok()
            //             {
            //                 // log::info!("Sending {:?}", file.path());
            //             }
            //         }
            //         _ => (),
            //     }
            // }
        });

        log::info!("After spawning file searcher...");
    }
}
