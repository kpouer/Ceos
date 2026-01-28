use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::line_group::DEFAULT_GROUP_SIZE;
use crate::ceos::buffer::line_group::LineGroup;
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use flate2::bufread::GzDecoder;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::Bound;
use std::ops::{Index, RangeBounds};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use crate::ceos::tools::misc_tool::gzip_uncompressed_size_fast;

#[derive(Debug)]
pub(crate) struct Buffer {
    pub(crate) path: Option<PathBuf>,
    /// the linegorups, the last one is never full. Eventually it is empty
    content: Vec<LineGroup>,
    length: usize,
    pub(crate) dirty: bool,
    pub(crate) sender: Sender<Event>,
}

const FILTERING: &str = "Filtering...";

impl Buffer {
    pub(crate) fn new(sender: Sender<Event>) -> Self {
        Self {
            path: None,
            content: vec![LineGroup::with_first_line(0)],
            length: 0,
            dirty: false,
            sender,
        }
    }

    pub(crate) fn new_from_string(sender: Sender<Event>, text: &str) -> Self {
        let mut buffer = Self::new(sender);

        let lines_iterator = text.lines();
        lines_iterator.into_iter().for_each(|line| {
            buffer.push_line(line);
        });

        buffer
    }

    pub(crate) fn new_from_file(
        path: PathBuf,
        sender: Sender<Event>,
    ) -> Result<Self, std::io::Error> {
        let mut buffer = Self {
            path: Some(path.clone()),
            ..Self::new(sender)
        };

        buffer.load_buffer(path)?;

        Ok(buffer)
    }

    fn load_buffer(&mut self, path: PathBuf) -> Result<(), io::Error> {
        let file = File::open(&path)?;

        let is_gz = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));

        let file_size = std::fs::metadata(&path)?.len() as usize;
        if is_gz {
            let file_size = match gzip_uncompressed_size_fast(&path) {
                Ok(size) => size as usize,
                Err(_) => file_size,
            };
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            let buffer_reader = io::BufReader::new(file);
            let decoder = GzDecoder::new(buffer_reader);
            let mut buffer_reader = io::BufReader::new(decoder);
            self.load_reader(path, file_size, &mut buffer_reader);
        } else {
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            let mut buffer_reader = io::BufReader::new(file);
            self.load_reader(path, file_size, &mut buffer_reader);
        }

        Ok(())
    }

    fn load_reader(&mut self, path: PathBuf, file_size: usize, buffer_reader: impl BufRead) {
        let mut start = Instant::now();
        for line_text in buffer_reader.lines().flatten() {
            self.push_line(line_text);
            if start.elapsed() > Duration::from_millis(50) {
                let _ = self
                    .sender
                    .send(BufferLoading(path.clone(), self.length, file_size));
                start = Instant::now();
            }
        }
    }

    /// Compress all line groups and free their in-memory lines to reclaim memory.
    /// This is primarily intended for debug/maintenance actions.
    pub(crate) fn compress_all_groups(&mut self) {
        for line_group in &mut self.content {
            if line_group.is_empty() {
                // Nothing to compress in an empty group
                continue;
            }

            if line_group.is_decompressed() {
                line_group.eventually_compress();
            }
            // Free lines if present; debug_assert in free() ensures it's compressed
            line_group.free();
        }
    }

    fn push_line(&mut self, line: impl Into<Line>) {
        let last_group = self.content.last_mut().expect("buffer is empty");
        let line = line.into();
        self.length += line.len() + 1;
        last_group.push(line);
        if last_group.is_full() {
            last_group.eventually_compress();
            last_group.free();
            let next_first = last_group.first_line() + last_group.line_count();
            self.content.push(LineGroup::with_first_line(next_first));
        }
    }

    pub(crate) fn line_groups(&self) -> &[LineGroup] {
        &self.content
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
            if let Some((group_index, line_index)) = self.find_group_index(global_index) {
                let line_group = &mut self.content[group_index];
                let max_in_group = line_group.len() - line_index;
                let take = to_remove.min(max_in_group);
                line_group.drain_lines(line_index..line_index + take);
                // If group is now empty, remove it to keep structure compact
                if line_group.is_empty() {
                    self.content.remove(group_index);
                }
                to_remove -= take;
                // global_index stays the same because we removed at this position
            } else {
                break;
            }
        }
        let new_length = self.compute_length();
        self.recompute_first_lines();
        self.dirty = true;
        new_length
    }

    pub(crate) fn filter_line_mut<F>(&mut self, filter: F) -> usize
    where
        F: FnMut(&mut Line) + Clone + Sync,
    {
        let _ = self.sender.send(Event::OperationStarted(
            FILTERING.to_owned(),
            self.content.len(),
        ));
        self.content.par_iter_mut().for_each(|line_group| {
            let _ = self
                .sender
                .send(Event::OperationIncrement(FILTERING.to_owned(), 1));
            line_group.filter_line_mut(filter.clone());
        });
        let new_length = self.compute_length();
        self.dirty = true;
        let _ = self
            .sender
            .send(Event::OperationFinished(FILTERING.to_owned()));
        new_length
    }

    pub(crate) fn retain_line_mut<F>(&mut self, filter: F) -> usize
    where
        F: Fn(&Line) -> bool + Sync + Send + Clone,
    {
        let _ = self.sender.send(Event::OperationStarted(
            FILTERING.to_owned(),
            self.content.len(),
        ));
        self.content.par_iter_mut().for_each(|line_group| {
            let _ = self
                .sender
                .send(Event::OperationIncrement(FILTERING.to_owned(), 1));
            line_group.retain(filter.clone());
        });
        // remove empty groups
        self.content.retain(|g| !g.is_empty());
        let new_length = self.compute_length();
        self.recompute_first_lines();
        self.dirty = true;
        let _ = self
            .sender
            .send(Event::OperationFinished(FILTERING.to_owned()));
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

        // Define a window to keep around the requested range to avoid thrashing.
        // Groups entirely outside this window will be recompressed.
        let window_start = start.saturating_sub(DEFAULT_GROUP_SIZE);
        let window_end = (end + DEFAULT_GROUP_SIZE).min(total_lines);

        // Walk groups and decompress those intersecting [start, end),
        // recompress those fully outside [window_start, window_end).
        let mut acc: usize = 0; // cumulative line count before current group
        for g in &mut self.content {
            let g_lines = g.line_count();
            let g_start = acc;
            let g_end = acc + g_lines;

            // check interval intersection with the exact read range
            if g_end > start && g_start < end {
                g.eventually_decompress();
            } else if g_end <= window_start || g_start >= window_end {
                g.eventually_compress();
                g.free();
            }

            acc = g_end;
            // do not early-break; we may need to compress groups after the end
        }
    }

    pub(crate) fn line_text(&self, line: usize) -> &str {
        let (gi, li) = self
            .find_group_index(line)
            .expect("line index out of bounds");
        self.content[gi][li].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.iter().map(|g| g.line_count()).sum()
    }

    /// Returns the buffer length.
    /// It is the number of chars + end of lines
    #[inline]
    pub(crate) const fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.content
            .iter()
            .map(|g| g.max_line_length())
            .max()
            .unwrap_or(0)
    }

    pub(crate) const fn group_count(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn compressed_group_count(&self) -> usize {
        self.content.iter().filter(|g| g.is_compressed()).count()
    }

    pub(crate) fn decompressed_group_count(&self) -> usize {
        self.content.iter().filter(|g| g.is_decompressed()).count()
    }

    pub(crate) fn decompressed_line_count(&self) -> usize {
        self.content
            .iter()
            .map(|g| g.decompressed_line_count())
            .sum()
    }

    pub(crate) fn compute_length(&mut self) -> usize {
        self.length = self
            .content
            .iter()
            .map(|line_group| line_group.len())
            .sum::<usize>()
            + self.content.len()
            - 1;
        self.length
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<LineGroup>>();
        let array_mem = self.content.capacity() * std::mem::size_of::<LineGroup>();
        let groups_mem: usize = self.content.iter().map(|line_group| line_group.mem()).sum();
        vec_overhead + array_mem + groups_mem
    }

    pub fn compressed_size(&self) -> usize {
        self.content.iter().map(|data| data.compressed_size()).sum()
    }

    fn recompute_first_lines(&mut self) {
        let mut first_line = 0;
        for g in &mut self.content {
            g.set_first_line(first_line);
            first_line += g.line_count();
        }
    }

    fn find_group_index(&self, mut line: usize) -> Option<(usize, usize)> {
        for (group_index, line_group) in self.content.iter().enumerate() {
            if line < line_group.line_count() {
                return Some((group_index, line));
            }
            line -= line_group.line_count();
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

impl Index<usize> for Buffer {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        let (group_index, line_index) = self
            .find_group_index(index)
            .expect("line index out of bounds");
        &self.content[group_index][line_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_builds_lines_and_lengths() {
        let (sender, _) = std::sync::mpsc::channel();
        let b = Buffer::new_from_string(sender, "a\nbb\nccc");
        assert_eq!(b.line_count(), 3);
        // Each line counted as len+1 in our model
        assert_eq!(b.len(), (1 + 1) + (2 + 1) + (3 + 1));
        assert_eq!(b.max_line_length(), 3);
        assert_eq!(b.line_text(0), "a");
        assert_eq!(b[1].content(), "bb");
    }

    #[test]
    fn iter_yields_all_lines_in_order() {
        let (sender, _) = std::sync::mpsc::channel();
        let b = Buffer::new_from_string(sender, "l1\nl2\nl3");
        let mut collected = Vec::new();
        b.line_groups().iter().for_each(|line_group| {
            line_group
                .lines()
                .iter()
                .map(|l| l.content().to_string())
                .for_each(|l| collected.push(l))
        });
        assert_eq!(collected, vec!["l1", "l2", "l3"]);
    }

    #[test]
    fn group_boundary_and_compression_path() {
        // Push exactly DEFAULT_GROUP_SIZE lines to trigger compression of first group
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new(sender);
        for i in 0..DEFAULT_GROUP_SIZE {
            b.push_line(format!("{:03}", i));
        }
        // We should still report correct counts and access
        assert_eq!(b.line_count(), DEFAULT_GROUP_SIZE);
        assert_eq!(b.max_line_length(), 3);
        // Access a few positions
        b.prepare_range_for_read(0..10);
        assert_eq!(b.line_text(0), "000");
        b.prepare_range_for_read(DEFAULT_GROUP_SIZE - 10..DEFAULT_GROUP_SIZE + 100);
        assert_eq!(
            b.line_text(DEFAULT_GROUP_SIZE - 1),
            format!("{:03}", DEFAULT_GROUP_SIZE - 1)
        );
    }

    #[test]
    fn filter_line_mut_updates_all_lines() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "a\nbb");
        let new_len = b.filter_line_mut(|line| {
            let mut s = line.to_string();
            s.push('x');
            *line = Line::from(s);
        });
        assert!(new_len >= b.len());
        assert!(b.line_text(0).ends_with('x'));
        assert!(b.line_text(1).ends_with('x'));
        assert!(b.dirty);
    }

    #[test]
    fn retain_line_mut_keeps_predicate_matches() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "a\nbb\nccc\ndddd");
        let _ = b.retain_line_mut(|l| l.len() % 2 == 0); // keep even lengths: 2 and 4
        assert_eq!(b.line_count(), 2);
        assert_eq!(b.line_text(0), "bb");
        assert_eq!(b.line_text(1), "dddd");
        assert!(b.dirty);
    }

    #[test]
    fn drain_line_mut_various_ranges() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "l0\nl1\nl2\nl3\nl4");
        let len1 = b.drain_line_mut(1..3); // remove l1,l2
        assert_eq!(b.line_count(), 3);
        assert_eq!(b.line_text(0), "l0");
        assert_eq!(b.line_text(1), "l3");
        assert_eq!(b.line_text(2), "l4");
        assert_eq!(len1, b.len());

        // Remove last element with inclusive range
        let _ = b.drain_line_mut(2..=2);
        assert_eq!(b.line_count(), 2);
        assert_eq!(b.line_text(1), "l3");
    }

    #[test]
    fn prepare_range_for_read_safe_and_accessible() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new(sender);
        for i in 0..(DEFAULT_GROUP_SIZE * 2 + 10) {
            b.push_line(format!("line{}", i));
        }
        // Should not panic and should allow access to middle range
        b.prepare_range_for_read(DEFAULT_GROUP_SIZE - 5..DEFAULT_GROUP_SIZE + 5);
        assert_eq!(
            b.line_text(DEFAULT_GROUP_SIZE),
            format!("line{}", DEFAULT_GROUP_SIZE)
        );
    }

    #[test]
    fn mem_non_decreasing_after_growth() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new(sender);
        let base = b.mem();
        b.push_line("abc");
        assert!(b.mem() >= base);
    }
}
