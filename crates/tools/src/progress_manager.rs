use std::collections::hash_map::Iter;
use std::collections::HashMap;

pub const BUFFER_LOADING: &str = "_BufferLoading_";

#[derive(Default)]
pub struct ProgressManager {
    pub progress: HashMap<String, Progress>,
}

impl ProgressManager {
    pub fn add(&mut self, id: String, label: String, max: usize) {
        self.progress.insert(
            id,
            Progress {
                label,
                current: 0,
                max,
            },
        );
    }

    pub fn update(&mut self, id: &str, current: usize) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current = current;
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.progress.remove(id);
    }

    pub fn is_empty(&self) -> bool {
        self.progress.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, String, Progress> {
        // todo : maybe not very efficient
        self.progress.iter()
    }
}

pub struct Progress {
    pub label: String,
    pub current: usize,
    pub max: usize,
}

impl Progress {
    pub fn percent(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}
