use std::collections::HashMap;
use std::collections::hash_map::Iter;
use crate::progress_operation::ProgressOperation;

#[derive(Default, Debug)]
pub(crate) struct ProgressManager {
    pub(crate) progress: HashMap<ProgressOperation, Progress>,
}

impl ProgressManager {
    pub(crate) fn add(&mut self, id: ProgressOperation, max: usize) {
        let label = id.to_string();
        self.progress.insert(
            id,
            Progress {
                label,
                current: 0,
                max,
            },
        );
    }

    pub(crate) fn update(&mut self, id: &ProgressOperation, current: usize) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current = current;
        }
    }

    pub(crate) fn increment(&mut self, id: &ProgressOperation, amount: usize) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current += amount;
        }
    }

    pub(crate) fn remove(&mut self, id: &ProgressOperation) {
        self.progress.remove(id);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.progress.is_empty()
    }

    pub(crate) fn iter(&self) -> Iter<'_, ProgressOperation, Progress> {
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
    pub(crate) const fn percent(&self) -> f32 {
        if self.current > self.max {
            1.0
        } else {
            self.current as f32 / self.max as f32
        }
    }
}
