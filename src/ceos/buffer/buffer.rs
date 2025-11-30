use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::line_group::LineGroup;
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::Bound;
use std::ops::{Index, RangeBounds};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct Buffer {
    pub(crate) path: Option<PathBuf>,
    /// the linegorups, the last one is never full. Eventually it is empty
    content: Vec<LineGroup>,
    length: usize,
    pub(crate) dirty: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            path: None,
            content: vec![LineGroup::default()],
            length: 0,
            dirty: false,
        }
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        let mut buffer = Self::default();

        let lines_iterator = text.lines();
        lines_iterator.into_iter().for_each(|line| {
            buffer.push_line(line);
        });

        buffer
    }
}

impl Buffer {
    pub(crate) fn new_from_file(
        path: PathBuf,
        sender: &Sender<Event>,
    ) -> Result<Self, std::io::Error> {
        let file_size = std::fs::metadata(&path)?.len() as usize;
        let _ = sender.send(BufferLoadingStarted(path.clone(), file_size));
        let file = File::open(&path)?;
        let buffer_reader = io::BufReader::new(file);
        let mut start = Instant::now();
        let mut buffer = Self {
            path: Some(path.clone()),
            ..Default::default()
        };
        for line_text in buffer_reader.lines().flatten() {
            buffer.push_line(line_text);
            if start.elapsed() > Duration::from_millis(50) {
                let _ = sender.send(BufferLoading(path.clone(), buffer.length, file_size));
                start = Instant::now();
            }
        }

        Ok(buffer)
    }

    fn push_line(&mut self, line: impl Into<Line>) {
        let last_group = self.content.last_mut().expect("buffer is empty");
        let line = line.into();
        self.length += line.len() + 1;
        last_group.push(line);
        if last_group.is_full() {
            last_group.compress();
            self.content.push(LineGroup::default());
        }
    }

    pub(crate) fn iter(&self) -> BufferIter<'_> {
        BufferIter::new(&self.content)
    }

    pub(crate) fn drain_line_mut<R>(&mut self, range: R) -> usize
    where
        R: RangeBounds<usize>,
    {
        // Convert RangeBounds to concrete start..end
        let (start, end) = self.normalize_range(range);
        if start >= end {
            return self.length;
        }

        let mut to_remove = end - start;
        let global_index = start;
        while to_remove > 0 {
            if let Some((gi, li)) = self.find_group_index(global_index) {
                let group_len = self.content[gi].len();
                let max_in_group = group_len - li;
                let take = to_remove.min(max_in_group);
                self.content[gi].drain(li..li + take);
                // If group is now empty, remove it to keep structure compact
                if self.content[gi].is_empty() {
                    self.content.remove(gi);
                }
                to_remove -= take;
                // global_index stays the same because we removed at this position
            } else {
                break;
            }
        }
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn filter_line_mut(&mut self, mut filter: impl FnMut(&mut Line)) -> usize {
        self.content
            .iter_mut()
            .for_each(|g| g.iter_mut().for_each(|l| filter(l)));
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn retain_line_mut(&mut self, mut filter: impl FnMut(&Line) -> bool) -> usize {
        self.content
            .iter_mut()
            .for_each(|g| g.retain(|l| filter(l)));
        // remove empty groups
        self.content.retain(|g| !g.is_empty());
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    /// Decompress only the line groups that intersect with the provided line range.
    /// This is a preparatory pass to ensure subsequent read operations on that
    /// span won't trigger on-demand decompression.
    ///
    /// The range is expressed in line indices (0-based, end-exclusive when Excluded/Unbounded).
    pub(crate) fn prepare_range_for_read<R: RangeBounds<usize>>(&mut self, range: R) {
        use std::ops::Bound;

        let total_lines = self.line_count();

        // Normalize start
        let mut start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s.saturating_add(1),
            Bound::Unbounded => 0,
        };

        // Normalize end
        let mut end = match range.end_bound() {
            Bound::Included(&e) => e.saturating_add(1),
            Bound::Excluded(&e) => e,
            Bound::Unbounded => total_lines,
        };

        // Clamp to valid bounds
        if start > total_lines {
            start = total_lines;
        }
        if end > total_lines {
            end = total_lines;
        }
        if start >= end {
            return;
        }

        // Walk groups and decompress those intersecting [start, end)
        let mut acc: usize = 0; // cumulative line count before current group
        for g in &mut self.content {
            let g_lines = g.line_count();
            let g_start = acc;
            let g_end = acc + g_lines;

            // check interval intersection
            if g_end > start && g_start < end {
                g.decompress();
            }

            acc = g_end;
            if acc >= end {
                break;
            }
        }
    }

    pub(crate) fn line_text(&self, line: usize) -> &str {
        let (gi, li) = self
            .find_group_index(line)
            .expect("line index out of bounds");
        &self.content[gi][li].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.iter().map(|g| g.line_count()).sum()
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.content
            .iter()
            .map(|g| g.max_line_length())
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn compute_length(&mut self) -> usize {
        self.length = self
            .content
            .iter()
            .flat_map(|g| g.iter())
            .map(|line| line.len())
            .sum::<usize>()
            + self.line_count();
        self.length
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<LineGroup>>();
        let array_mem = self.content.capacity() * std::mem::size_of::<LineGroup>();
        let groups_mem: usize = self.content.iter().map(|g| g.mem()).sum();
        vec_overhead + array_mem + groups_mem
    }
}

impl Index<usize> for Buffer {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        let (gi, li) = self
            .find_group_index(index)
            .expect("line index out of bounds");
        &self.content[gi][li]
    }
}

impl Buffer {
    fn find_group_index(&self, mut line: usize) -> Option<(usize, usize)> {
        for (gi, g) in self.content.iter().enumerate() {
            if line < g.line_count() {
                return Some((gi, line));
            }
            line -= g.line_count();
        }
        None
    }

    fn normalize_range<R: RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        let start = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.line_count(),
        };
        (start.min(self.line_count()), end.min(self.line_count()))
    }
}

pub(crate) struct BufferIter<'a> {
    groups: &'a [LineGroup],
    gi: usize,
    li: usize,
}

impl<'a> BufferIter<'a> {
    fn new(groups: &'a [LineGroup]) -> Self {
        Self {
            groups,
            gi: 0,
            li: 0,
        }
    }
}

impl<'a> Iterator for BufferIter<'a> {
    type Item = &'a Line;

    fn next(&mut self) -> Option<Self::Item> {
        while self.gi < self.groups.len() {
            let g = &self.groups[self.gi];
            if self.li < g.len() {
                let item = &g[self.li];
                self.li += 1;
                return Some(item);
            } else {
                self.gi += 1;
                self.li = 0;
            }
        }
        None
    }
}
