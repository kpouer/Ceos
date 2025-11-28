use crate::ceos::buffer::line::Line;
use std::ops::RangeBounds;
use std::ops::Index;
use log::warn;

pub(crate) const DEFAULT_GROUP_SIZE: usize = 1000;

#[derive(Debug)]
pub(crate) struct LineGroup {
    lines: Vec<Line>,
}

impl Default for LineGroup {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_GROUP_SIZE)
    }
}

impl LineGroup {
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self { lines: Vec::with_capacity(cap) }
    }

    pub(crate) fn is_full(&self) -> bool { 
        self.lines.len() >= DEFAULT_GROUP_SIZE
    }

    pub(crate) fn compress(&self) {
        warn!("Not yet implemented: compressing line group")
    }

    pub(crate) fn push(&mut self, line: Line) {
        self.lines.push(line)
    }

    pub(crate) fn len(&self) -> usize { self.lines.len() }

    pub(crate) fn is_empty(&self) -> bool { self.lines.is_empty() }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Line> { self.lines.iter() }

    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, Line> { self.lines.iter_mut() }

    pub(crate) fn retain<F: FnMut(&Line) -> bool>(&mut self, f: F) { self.lines.retain(f) }

    pub(crate) fn drain<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    { self.lines.drain(range); }

    pub(crate) fn max_line_length(&self) -> usize {
        self.lines.iter().map(|l| l.len()).max().unwrap_or(0)
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<Line>>();
        let array_mem = self.lines.capacity() * std::mem::size_of::<Line>();
        let strings_mem: usize = self.lines.iter().map(|l| l.mem()).sum();
        vec_overhead + array_mem + strings_mem
    }
}

impl Index<usize> for LineGroup {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}
