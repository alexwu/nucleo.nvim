pub mod files;
pub mod diagnostics;

pub trait Source {
    fn should_preview(&self) -> bool;
}
