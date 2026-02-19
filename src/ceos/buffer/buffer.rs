use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::line_group::LineGroup;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::tools::misc_tool::{gzip_uncompressed_size_fast, is_gzip};
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use flate2::bufread::GzDecoder;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::{Bound, Index, RangeBounds};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use log::{info, warn};

const DEFAULT_GROUP_SIZE: usize = 1000;

#[derive(Debug)]
pub(crate) struct Buffer {
    pub(crate) path: Option<PathBuf>,
    /// the linegroups, the last one is never full. Eventually it is empty
    content: Vec<LineGroup>,
    length: usize,
    pub(crate) dirty: bool,
    pub(crate) sender: Sender<Event>,
    /// The size of the groups used for line compression.
    group_size: usize,
}

const FILTERING: &str = "Filtering...";

impl Buffer {
    pub(crate) fn new_empty_buffer(sender: Sender<Event>) -> Self {
        let mut buffer = Self::new_with_group_size(sender, DEFAULT_GROUP_SIZE);
        buffer.push_line("");
        buffer
    }

    pub(crate) fn new_from_string(sender: Sender<Event>, text: &str, group_size: usize) -> Self {
        let mut buffer = Self::new_with_group_size(sender, group_size);

        let lines_iterator = text.lines();
        lines_iterator.into_iter().for_each(|line| {
            buffer.push_line(line);
        });

        buffer
    }

    fn new_with_group_size(sender: Sender<Event>, group_size: usize) -> Self {
        Self {
            path: None,
            content: vec![LineGroup::new(0, group_size)],
            length: 0,
            dirty: false,
            sender,
            group_size,
        }
    }

    pub(crate) fn new_from_file(
        path: PathBuf,
        sender: Sender<Event>,
    ) -> Result<Self, std::io::Error> {
        let mut buffer = Self {
            path: Some(path),
            ..Self::new_with_group_size(sender, DEFAULT_GROUP_SIZE)
        };

        buffer.load_buffer()?;

        Ok(buffer)
    }

    pub(crate) fn set_path(&mut self, path: PathBuf) {
        info!("set path to {path:?}");
        self.path = Some(path);
        self.dirty = true;
    }

    fn load_buffer(&mut self) -> Result<(), io::Error> {
        let path = self.path.as_ref().expect("buffer has no path");
        let file = File::open(path)?;

        let mut buffer_reader = io::BufReader::new(file);

        let file_size = std::fs::metadata(path)?.len() as usize;
        if is_gzip(&mut buffer_reader) {
            let file_size = match gzip_uncompressed_size_fast(path) {
                Ok(size) => size as usize,
                Err(_) => file_size,
            };
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            let decoder = GzDecoder::new(buffer_reader);
            let mut buffer_reader = io::BufReader::new(decoder);
            self.load_reader(file_size, &mut buffer_reader)?;
        } else {
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            self.load_reader(file_size, &mut buffer_reader)?;
        }

        Ok(())
    }

    fn load_reader(
        &mut self,
        file_size: usize,
        buffer_reader: impl BufRead,
    ) -> Result<(), io::Error> {
        let mut start = Instant::now();
        for line_text in buffer_reader.lines() {
            self.push_line(line_text?);
            if start.elapsed() > Duration::from_millis(50) {
                let path = self.path.clone().expect("buffer has no path");
                let _ = self
                    .sender
                    .send(BufferLoading(path, self.length, file_size));
                start = Instant::now();
            }
        }
        Ok(())
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
            self.content
                .push(LineGroup::new(next_first, self.group_size));
        }
    }

    /// Delete a range of text from the buffer.
    pub(crate) fn delete_range(&mut self, text_range: TextRange) {
        let line_count = self.line_count();
        if line_count == 0 || text_range.start_line >= line_count || text_range.is_empty() {
            return;
        }

        let start_line = text_range.start_line;
        let end_line = text_range.end_line.min(line_count.saturating_sub(1));

        if start_line == end_line {
            if let Some((group_index, line_in_group)) = self.find_group_index(start_line) {
                let line_group = &mut self.content[group_index];
                line_group.filter_line_mut(line_in_group, |line| {
                    line.drain(text_range.start_column..text_range.end_column);
                });
            }
        } else {
            self.delete_across_lines(
                start_line,
                text_range.start_column,
                end_line,
                text_range.end_column,
            );
        }

        self.compute_length();
        self.recompute_first_lines();
        self.dirty = true;
    }

    fn delete_across_lines(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) {
        let Some((start_group_index, start_line_in_group)) = self
            .find_group_index(start_line) else {
            warn!("start_line out of bounds");
            return;
        };
        let Some((end_group_index, end_line_in_group)) = self
            .find_group_index(end_line) else {
            warn!("end_line out of bounds");
            return;
        };

        if start_group_index == end_group_index {
            info!("start group and end group are the same");

            let line_group = &mut self.content[start_group_index];
            line_group.eventually_decompress();

            let suffix = line_group[end_line_in_group].content()[end_col..].to_owned();
            line_group.filter_line_mut(start_line_in_group, |line| {
                line.drain(start_col..);
                line.push_str(&suffix);
            });

            line_group.drain_lines(start_line_in_group + 1..=end_line_in_group);
            return;
        }

        let suffix = {
            // process the end group and retrieve the suffix
            let end_group = &mut self.content[end_group_index];
            let last_line = &end_group.lines()[end_line_in_group];
            let suffix = last_line.content()[end_col..].to_owned();
            end_group.drain_lines(0..=end_line_in_group);
            suffix
        };

        let first_group = &mut self.content[start_group_index];
        first_group.filter_line_mut(start_line_in_group, |line| {
            line.drain(start_col..);
            line.push_str(&suffix);
        });
        first_group.drain_lines(start_line_in_group + 1..);

        // drain the linegroups between the start and the end group
        if start_group_index + 1 < end_group_index {
            self.content.drain(start_group_index + 1..end_group_index);
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
        let (start_line, end_line) = self.normalize_range(range);
        if start_line >= end_line {
            return self.compute_length();
        }

        let (start_group_index, start_line_in_group) = self
            .find_group_index(start_line)
            .expect("start_line out of bounds");
        let (end_group_index, end_line_in_group) = self
            .find_group_index(end_line - 1)
            .expect("end_line out of bounds");

        let start_group_line_count = self.content[start_group_index].line_count();
        let should_remove_first_group = start_line_in_group == 0
            && (start_group_index < end_group_index
                || end_line_in_group == start_group_line_count - 1);

        if !should_remove_first_group {
            let end_in_group = if start_group_index == end_group_index {
                end_line_in_group + 1
            } else {
                start_group_line_count
            };
            self.content[start_group_index].drain_lines(start_line_in_group..end_in_group);
        }

        if start_group_index != end_group_index {
            let first_group_to_remove = if should_remove_first_group {
                start_group_index
            } else {
                start_group_index + 1
            };
            let end_group = &mut self.content[end_group_index];
            if end_line_in_group == end_group.line_count() - 1 {
                // all the last group has to be removed
                self.content.drain(first_group_to_remove..=end_group_index);
            } else {
                end_group.drain_lines(..=end_line_in_group);
                self.content.drain(first_group_to_remove..end_group_index);
            }
        } else if should_remove_first_group {
            self.content.remove(start_group_index);
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
            line_group.filter_lines_mut(filter.clone());
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
        let window_start = start.saturating_sub(self.group_size);
        let window_end = (end + self.group_size).min(total_lines);

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

    pub(crate) fn line_length(&self, line: usize) -> usize {
        if let Some((gi, li)) = self.find_group_index(line) {
            return self.content[gi][li].len();
        }
        0
    }

    /// Returns the text of the line at the given index.
    /// The given index is 0-based
    pub(crate) fn line_text(&self, line: usize) -> &str {
        let (gi, li) = self
            .find_group_index(line)
            .expect("line index out of bounds");
        self.content[gi][li].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.iter().map(|g| g.line_count()).sum()
    }

    pub(crate) fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        if ch == '\n' {
            self.insert_newline(line, col);
            return;
        }
        if let Some((gi, _)) = self.find_group_index(line) {
            let relative_line = line - self.content[gi].first_line();
            self.content[gi].filter_line_mut(relative_line, |l| {
                l.insert(col, ch);
            });
            self.compute_length();
            self.dirty = true;
        }
    }

    pub(crate) fn insert_newline(&mut self, line: usize, col: usize) {
        if let Some((gi, li)) = self.find_group_index(line) {
            let suffix = {
                let line_group = &mut self.content[gi];
                let mut suffix = String::new();
                line_group.filter_line_mut(li, |l| {
                    suffix = l.content()[col..].to_owned();
                    l.drain(col..);
                });
                suffix
            };

            let line_group = &mut self.content[gi];
            line_group.insert_line(li + 1, Line::from(suffix));

            self.compute_length();
            self.recompute_first_lines();
            self.dirty = true;
        }
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
            .sum::<usize>();
        self.length
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<LineGroup>>();
        let array_mem = self.content.capacity() * std::mem::size_of::<LineGroup>();
        let groups_mem: usize = self.content.iter().map(|line_group| line_group.mem()).sum();
        vec_overhead + array_mem + groups_mem
    }

    pub(crate) fn compressed_size(&self) -> usize {
        self.content.iter().map(|data| data.compressed_size()).sum()
    }

    fn recompute_first_lines(&mut self) {
        let mut first_line = 0;
        for g in &mut self.content {
            g.set_first_line(first_line);
            first_line += g.line_count();
        }
    }

    /// Finds the index of the group and the corresponding line within that group,
    /// given a line number in the aggregated content.
    ///
    /// # Arguments
    ///
    /// * `line` - The zero-based line number in the aggregated content to locate.
    ///
    /// # Returns
    ///
    /// * `Some((usize, usize))` - A tuple containing:
    ///   - The index of the group (`usize`).
    ///   - The corresponding line number relative to the group (`usize`).
    /// * `None` - If the given line number exceeds the total number of lines
    ///   in all groups combined.
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
        // todo : use let Range { start, end } = slice::range(range, ..len);
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

    #[cfg(debug_assertions)]
    fn debug(&self) {
        println!("Buffer Debug Info:");
        println!("Line Count: {}", self.line_count());
        println!("Dirty: {}", self.dirty);
        println!("Content:");
        for line_group in &self.content {
            line_group.debug();
        }
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
        let mut b = Buffer::new_from_string(sender, "a\nbb\nccc", 2);
        assert_eq!(b.line_count(), 3);
        // Each line counted as len+1 in our model
        assert_eq!(b.len(), (1 + 1) + (2 + 1) + (3 + 1));
        assert_eq!(b.max_line_length(), 3);
        b.prepare_range_for_read(..);
        assert_eq!(b.line_text(0), "a");
        assert_eq!(b[1].content(), "bb");
    }

    #[test]
    fn iter_yields_all_lines_in_order() {
        let (sender, _) = std::sync::mpsc::channel();
        let b = Buffer::new_from_string(sender, "l1\nl2\nl3", 2);
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
        let mut b = Buffer::new_with_group_size(sender, 2);
        for i in 0..b.group_size {
            b.push_line(format!("{:03}", i));
        }
        // We should still report correct counts and access
        assert_eq!(b.line_count(), b.group_size);
        assert_eq!(b.max_line_length(), 3);
        // Access a few positions
        b.prepare_range_for_read(0..10);
        assert_eq!(b.line_text(0), "000");
        let start = if b.group_size > 10 {
            b.group_size - 10
        } else {
            0
        };
        b.prepare_range_for_read(start..b.group_size + 100);
        assert_eq!(
            b.line_text(b.group_size - 1),
            format!("{:03}", b.group_size - 1)
        );
    }

    #[test]
    fn filter_line_mut_updates_all_lines() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "a\nbb", 2);
        let new_len = b.filter_line_mut(|line| {
            let mut s = line.to_string();
            s.push('x');
            *line = Line::from(s);
        });
        assert!(new_len >= b.len());
        b.prepare_range_for_read(..);
        assert!(b.line_text(0).ends_with('x'));
        assert!(b.line_text(1).ends_with('x'));
        assert!(b.dirty);
    }

    #[test]
    fn retain_line_mut_keeps_predicate_matches() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "a\nbb\nccc\ndddd", 2);
        let _ = b.retain_line_mut(|l| l.len() % 2 == 0); // keep even lengths: 2 and 4
        assert_eq!(b.line_count(), 2);
        assert_eq!(b.line_text(0), "bb");
        assert_eq!(b.line_text(1), "dddd");
        assert!(b.dirty);
    }

    #[test]
    fn drain_line_mut_various_ranges() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut buffer = Buffer::new_from_string(sender.clone(), "l0\nl1\nl2\nl3\nl4", 2);
        // buffer has 5 lines: ["l0", "l1", "l2", "l3", "l4"]
        // group_size = 2
        // groups: G0: [l0, l1], G1: [l2, l3], G2: [l4]

        let _ = buffer.drain_line_mut(1..3); // remove l1, l2
        // expected: ["l0", "l3", "l4"]
        // groups: G0: [l0], G1: [l3], G2: [l4] (or merged, but drain_line_mut doesn't merge)
        buffer.debug();
        assert_eq!(buffer.line_count(), 3);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l3");
        assert_eq!(buffer.line_text(2), "l4");

        // Remove last element with inclusive range
        let mut buffer = Buffer::new_from_string(sender.clone(), "l0\nl1\nl2", 2);
        let _ = buffer.drain_line_mut(2..=2);
        assert_eq!(buffer.line_count(), 2);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l1");

        // Remove all lines
        let mut buffer = Buffer::new_from_string(sender, "l0\nl1\nl2", 2);
        let _ = buffer.drain_line_mut(..);
        assert_eq!(buffer.line_count(), 0);
    }

    #[test]
    fn new_from_file_loads_cargo_toml() {
        let (sender, _) = std::sync::mpsc::channel();
        let path = PathBuf::from("Cargo.toml");
        let mut buffer = Buffer::new_from_file(path, sender).expect("Failed to load Cargo.toml");

        assert!(buffer.line_count() > 0);
        let first_line = buffer.line_text(0);
        assert!(first_line.contains("[package]"));
        buffer.compress_all_groups();
    }

    #[test]
    fn drain_line_mut_bug_reproduction() {
        let (sender, _) = std::sync::mpsc::channel();

        // Test 1: Drain spanning multiple groups
        let mut buffer = Buffer::new_from_string(sender.clone(), "l0\nl1\nl2\nl3\nl4", 2);
        // Groups: G0:[l0, l1], G1:[l2, l3], G2:[l4]
        // Drain 1..4 (l1, l2, l3)
        buffer.drain_line_mut(1..4);
        // Expected: ["l0", "l4"]
        assert_eq!(
            buffer.line_count(),
            2,
            "Line count should be 2 after draining 1..4"
        );
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l0");
        assert_eq!(buffer.line_text(1), "l4");

        // Test 2: should_remove_first_group when spanning multiple groups
        let mut buffer = Buffer::new_from_string(sender, "l0\nl1\nl2\nl3", 2);
        // Groups: G0:[l0, l1], G1:[l2, l3]
        // Drain 0..3 (l0, l1, l2)
        // start_line_in_group = 0
        // lines_to_delete = 3
        // line_group.line_count() = 2
        // current code's should_remove_first_group = (0 == 0 && 2 == 3) => false
        // BUT it SHOULD remove G0 because l0, l1 are both being deleted.
        buffer.drain_line_mut(0..3);
        assert_eq!(buffer.line_count(), 1);
        buffer.prepare_range_for_read(..);
        assert_eq!(buffer.line_text(0), "l3");
    }

    #[test]
    fn prepare_range_for_read_safe_and_accessible() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_with_group_size(sender, 2);
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
    fn delete_range_single_line() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "abcdef", 2);
        b.delete_range(TextRange::new(0, 2, 0, 5));
        assert_eq!(b.line_text(0), "abf");
        assert_eq!(b.line_count(), 1);
        assert!(b.dirty);
    }

    #[test]
    fn delete_range_multi_line_merges() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_from_string(sender, "hello\nworld\n!!!", 2);
        b.delete_range(TextRange::new(0, 2, 1, 3));
        assert_eq!(b.line_text(0), "held");
        assert_eq!(b.line_text(1), "!!!");
        assert_eq!(b.line_count(), 2);
        assert!(b.dirty);
    }

    #[test]
    fn delete_range_to_line_start() {
        let (sender, _) = std::sync::mpsc::channel();
        const TEXT: &str = "aaa\n\
        bbb\n\
        ccc";
        let mut buffer = Buffer::new_from_string(sender, TEXT, 2);
        let range = TextRange::new(0, 1, 2, 1);
        buffer.delete_range(range);
        buffer.debug();
        assert_eq!(buffer.line_text(0), "acc");
        assert_eq!(buffer.line_count(), 1);
        assert!(buffer.dirty);
    }

    #[test]
    fn mem_non_decreasing_after_growth() {
        let (sender, _) = std::sync::mpsc::channel();
        let mut b = Buffer::new_with_group_size(sender, 2);
        let base = b.mem();
        b.push_line("abc");
        assert!(b.mem() >= base);
    }
}
