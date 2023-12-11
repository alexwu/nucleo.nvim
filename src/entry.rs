use std::path::Path;
use std::fmt::Debug;

use nucleo::Utf32String;
use serde::{Deserialize, Serialize};

pub trait Entry: for<'a> Deserialize<'a> + Debug + Serialize + Clone + Sync + Send + 'static {
    fn from_path(path: &Path, cwd: Option<String>) -> Self;

    fn display(&self) -> String;
    fn into_utf32(self) -> Utf32String;
    fn with_indices(self, indices: Vec<(u32, u32)>) -> Self;
    fn with_selected(self, selected: bool) -> Self;
}
