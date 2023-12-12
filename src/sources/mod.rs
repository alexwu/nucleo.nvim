pub mod files;

pub trait Source {
    fn should_preview(&self) -> bool;
}
