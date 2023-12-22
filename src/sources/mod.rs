pub mod diagnostics;
pub mod files;
pub mod git;

pub trait Source {
    fn should_preview(&self) -> bool;
}
