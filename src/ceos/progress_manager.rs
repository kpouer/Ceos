use std::collections::HashMap;
use std::collections::hash_map::Iter;

pub(crate) const BUFFER_LOADING: &str = "_BufferLoading_";
pub(crate) const BUFFER_SAVING: &str = "_BufferSaving_";

#[derive(Default, Debug)]
pub(crate) struct ProgressManager {
    pub(crate) progress: HashMap<String, Progress>,
}

impl ProgressManager {
    pub(crate) fn add(&mut self, id: String, label: String, max: usize) {
        self.progress.insert(
            id,
            Progress {
                label,
                current: 0,
                max,
            },
        );
    }

    pub(crate) fn update(&mut self, id: &str, current: usize) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current = current;
        }
    }

    pub(crate) fn increment(&mut self, id: &str) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current += 1;
        }
    }

    pub(crate) fn remove(&mut self, id: &str) {
        self.progress.remove(id);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.progress.is_empty()
    }

    pub(crate) fn iter(&self) -> Iter<'_, String, Progress> {
        // todo : maybe not very efficient
        self.progress.iter()
    }
}

#[derive(Debug)]
pub(crate) struct Progress {
    pub(crate) label: String,
    pub(crate) current: usize,
    pub(crate) max: usize,
}

impl Progress {
    pub(crate) fn percent(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}
