use crate::progress_operation::ProgressOperation;
use num_traits::{ToPrimitive, Zero};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::fmt::Debug;
use std::ops::AddAssign;

#[derive(Default, Debug)]
pub(crate) struct ProgressManager<T>
where
    T: PartialOrd + ToPrimitive + Zero + Default + AddAssign + Debug,
{
    pub(crate) progress: HashMap<ProgressOperation, Progress<T>>,
}

impl<T> ProgressManager<T>
where
    T: PartialOrd + ToPrimitive + Zero + Default + AddAssign + Debug,
{
    pub(crate) fn add(&mut self, id: ProgressOperation, max: T) {
        let label = id.to_string();
        self.progress.insert(id, Progress::<T>::new(label, max));
    }

    pub(crate) fn update(&mut self, id: &ProgressOperation, current: T) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current = current;
        }
    }

    pub(crate) fn remove(&mut self, id: &ProgressOperation) {
        self.progress.remove(id);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.progress.is_empty()
    }

    pub(crate) fn iter(&self) -> Iter<'_, ProgressOperation, Progress<T>> {
        // todo : maybe not very efficient
        self.progress.iter()
    }

    pub(crate) fn increment(&mut self, id: &ProgressOperation, amount: T) {
        if let Some(progress) = self.progress.get_mut(id) {
            progress.current += amount;
        }
    }
}

#[derive(Debug)]
pub(crate) struct Progress<T> {
    pub(crate) label: String,
    pub(crate) current: T,
    pub(crate) max: T,
}

impl<T: Default> Progress<T>
where
    T: PartialOrd + ToPrimitive + Zero + Default,
{
    fn new(label: String, max: T) -> Self {
        Self {
            label,
            current: T::zero(),
            max,
        }
    }

    pub(crate) fn percent(&self) -> f32 {
        if self.current.is_zero() {
            return 0.0;
        }
        if self.max.is_zero() || self.current >= self.max {
            return 1.0;
        }
        let current: f32 = self.current.to_f32().unwrap_or(0.0);
        let max: f32 = self.max.to_f32().unwrap_or(1.0);
        current / max
    }
}
