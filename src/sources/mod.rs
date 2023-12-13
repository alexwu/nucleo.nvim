pub mod files;
pub mod lsp;

pub trait Source {
    fn should_preview(&self) -> bool;
}
