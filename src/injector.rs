use std::{path::Path, sync::mpsc};

use ignore::{types::TypesBuilder, DirEntry, WalkBuilder, WalkState};
use nucleo::Utf32String;
use tokio::runtime::Runtime;

pub struct Injector<T: Clone + Into<Utf32String>>(nucleo::Injector<T>);

impl<T: Clone + Into<Utf32String>> From<nucleo::Injector<T>> for Injector<T> {
    fn from(value: nucleo::Injector<T>) -> Self {
        Self(value)
    }
}

impl<T: Clone + Into<Utf32String>> Clone for Injector<T> {
    fn clone(&self) -> Self {
        <nucleo::Injector<T> as Clone>::clone(&self.0).into()
    }
}

impl Injector<String> {
    pub fn push(&self, value: String, fill_columns: impl FnOnce(&mut [Utf32String])) -> u32 {
        self.0.push(value, fill_columns)
    }

    pub fn populate_files(self, cwd: String, git_ignore: bool) {
        log::info!("Populating picker with {}", &cwd);
        let runtime = Runtime::new().expect("Failed to create runtime");

        let (tx, rx) = mpsc::channel::<String>();
        let _add_to_injector_thread = std::thread::spawn(move || -> anyhow::Result<()> {
            for val in rx.iter() {
                self.push(val.clone(), |dst| dst[0] = val.into());
            }
            Ok(())
        });

        let _ = runtime.spawn(async move {
            let dir = Path::new(&cwd);
            log::info!("Spawning file searcher...");
            let mut walk_builder = WalkBuilder::new(dir.clone());
            walk_builder
                .hidden(true)
                .follow_links(true)
                .git_ignore(git_ignore)
                .sort_by_file_name(|name1, name2| name1.cmp(name2));
            let mut type_builder = TypesBuilder::new();
            type_builder
                .add(
                    "compressed",
                    "*.{zip,gz,bz2,zst,lzo,sz,tgz,tbz2,lz,lz4,lzma,lzo,z,Z,xz,7z,rar,cab}",
                )
                .expect("Invalid type definition");
            type_builder.negate("all");
            let excluded_types = type_builder
                .build()
                .expect("failed to build excluded_types");
            walk_builder.types(excluded_types);
            walk_builder.build_parallel().run(|| {
                let cwd = cwd.clone();
                let tx = tx.clone();
                Box::new(move |path: Result<DirEntry, ignore::Error>| -> WalkState {
                    match path {
                        Ok(file) if file.path().is_file() => {
                            let val = file
                                .path()
                                .strip_prefix(&cwd)
                                .expect("Failed to strip prefix")
                                .to_str()
                                .expect("Failed to convert path to string")
                                .to_string();
                            // log::info!("Adding {}", &val);

                            match tx.send(val.clone()) {
                                Ok(_) => WalkState::Continue,
                                Err(_) => WalkState::Skip,
                            }
                        }
                        Ok(_) => WalkState::Continue,
                        Err(_) => WalkState::Skip,
                    }
                })
            });
        });

        log::info!("After spawning file searcher...");
    }
}

pub fn populate_injector(injector: Injector<String>, cwd: String, git_ignore: bool) {
    log::info!("Populating picker with {}", &cwd);
    let runtime = Runtime::new().expect("Failed to create runtime");

    let (tx, rx) = mpsc::channel::<String>();
    let _add_to_injector_thread = std::thread::spawn(move || -> anyhow::Result<()> {
        for val in rx.iter() {
            injector.push(val.clone(), |dst| dst[0] = val.into());
        }
        Ok(())
    });

    let _ = runtime.spawn(async move {
        let dir = Path::new(&cwd);
        log::info!("Spawning file searcher...");
        let mut walk_builder = WalkBuilder::new(dir.clone());
        walk_builder
            .hidden(true)
            .follow_links(true)
            .git_ignore(git_ignore)
            .sort_by_file_name(|name1, name2| name1.cmp(name2));
        let mut type_builder = TypesBuilder::new();
        type_builder
            .add(
                "compressed",
                "*.{zip,gz,bz2,zst,lzo,sz,tgz,tbz2,lz,lz4,lzma,lzo,z,Z,xz,7z,rar,cab}",
            )
            .expect("Invalid type definition");
        type_builder.negate("all");
        let excluded_types = type_builder
            .build()
            .expect("failed to build excluded_types");
        walk_builder.types(excluded_types);
        walk_builder.build_parallel().run(|| {
            let cwd = cwd.clone();
            let tx = tx.clone();
            Box::new(move |path: Result<DirEntry, ignore::Error>| -> WalkState {
                match path {
                    Ok(file) if file.path().is_file() => {
                        let val = file
                            .path()
                            .strip_prefix(&cwd)
                            .expect("Failed to strip prefix")
                            .to_str()
                            .expect("Failed to convert path to string")
                            .to_string();
                        log::info!("Adding {}", &val);

                        match tx.send(val.clone()) {
                            Ok(_) => WalkState::Continue,
                            Err(_) => WalkState::Skip,
                        }
                    }
                    Ok(_) => WalkState::Continue,
                    Err(_) => WalkState::Skip,
                }
            })
        });
    });

    log::info!("After spawning file searcher...");
}
