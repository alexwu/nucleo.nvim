#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Lua(#[from] mlua::Error),
    #[error("{0}")]
    TryRecv(#[from] crossbeam_channel::TryRecvError),
    #[error("{0}")]
    Git(#[from] git2::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
