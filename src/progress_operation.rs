use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ProgressOperation {
    Filtering,
    Searching,
    BufferLoading(Option<PathBuf>),
    BufferSaving(Option<PathBuf>),
}

impl Display for ProgressOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressOperation::Filtering => write!(f, "Filtering ..."),
            ProgressOperation::Searching => write!(f, "Searching ..."),
            ProgressOperation::BufferLoading(path) => {
                if let Some(path) = path {
                    write!(f, "Loading {path:?}")
                } else {
                    write!(f, "Loading ...")
                }
            }
            ProgressOperation::BufferSaving(path) => {
                if let Some(path) = path {
                    write!(f, "Saving {path:?}")
                } else {
                    write!(f, "Saving ...")
                }
            }
        }
    }
}

impl PartialEq for ProgressOperation {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ProgressOperation::Filtering, ProgressOperation::Filtering)
                | (ProgressOperation::Searching, ProgressOperation::Searching)
                | (
                    ProgressOperation::BufferLoading(_),
                    ProgressOperation::BufferLoading(_)
                )
                | (
                    ProgressOperation::BufferSaving(_),
                    ProgressOperation::BufferSaving(_)
                )
        )
    }
}

impl Eq for ProgressOperation {}

impl Hash for ProgressOperation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let disc = match self {
            ProgressOperation::Filtering => 0,
            ProgressOperation::Searching => 1,
            ProgressOperation::BufferLoading(_) => 2,
            ProgressOperation::BufferSaving(_) => 3,
        };
        disc.hash(state);
    }
}
