use std::path::Path;

use nucleo::Utf32String;
use serde::{Deserialize, Serialize};

pub trait Entry: for<'a> Deserialize<'a> + Serialize + Clone + Sync + Send + 'static {
    fn into_utf32(self) -> Utf32String;
    fn from_path(path: &Path, cwd: Option<String>) -> Self;
    fn set_selected(&mut self, selected: bool);
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
}
