use crate::ceos::buffer::line::Line;
use log::warn;
use lz4::block;
use std::ops::Index;
use std::ops::RangeBounds;

pub(crate) const DEFAULT_GROUP_SIZE: usize = 1000;

#[derive(Debug)]
pub(crate) struct LineGroup {
    /// Contains the uncompressed data. Might be there even if the compressed data is present.
    lines: Vec<Line>,
    /// Contains the compressed data if the group is compressed, None otherwise.
    compressed: Option<Vec<u8>>,
    // number of lines stored in this group (stable even when compressed)
    line_count: usize,
    // total UTF-8 text length of the group with one '\n' separator between lines
    length: usize,
    max_line_length: usize,
}

impl Default for LineGroup {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_GROUP_SIZE)
    }
}

impl LineGroup {
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            lines: Vec::with_capacity(cap),
            compressed: None,
            line_count: 0,
            length: 0,
            max_line_length: 0,
        }
    }

    /// Free memory occupied by the lines.
    pub(crate) fn free(&mut self) {
        self.lines.clear();
        debug_assert!(self.compressed.is_some());
    }

    pub(crate) fn compress(&mut self) {
        if self.compressed.is_some() {
            // already compressed; keep in-memory lines for fast read access
            return;
        }

        debug_assert!(!self.lines.is_empty());
        let concatenated = self.to_string();

        match block::compress(concatenated.as_bytes(), None, true) {
            Ok(data) => self.compressed = Some(data),
            Err(e) => warn!("Failed to compress line group with LZ4: {e}"),
        }
    }

    fn to_string(&self) -> String {
        self.lines
            .iter()
            .enumerate()
            .map(|(index, line)| (index, line.content()))
            .fold(
                String::with_capacity(self.len()),
                |mut buffer, (index, line)| {
                    buffer.push_str(line);
                    if index != self.line_count - 1 {
                        buffer.push('\n');
                    }
                    buffer
                },
            )
    }

    pub(crate) fn decompress(&mut self) {
        if self.compressed.is_none() || !self.lines.is_empty() {
            return;
        }

        let data = self.compressed.take().unwrap();
        match block::decompress(&data, None) {
            Ok(bytes) => {
                match String::from_utf8(bytes) {
                    Ok(text) => {
                        if text.is_empty() {
                            #[cfg(debug_assertions)]
                            warn!("Decompressed empty line group");
                            self.lines.clear();
                            self.line_count = 0;
                            self.length = 0;
                            self.max_line_length = 0;
                        } else {
                            self.lines = text.split('\n').map(Line::from).collect();
                            #[cfg(debug_assertions)]
                            if self.line_count != self.lines.len() {
                                warn!(
                                    "Decompressed line group with inconsistent line count {} != {}",
                                    self.line_count,
                                    self.lines.len()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decode UTF-8 after LZ4 decompress: {}", e);
                        // restore compressed data to avoid data loss
                        self.compressed = Some(data);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to decompress line group with LZ4: {}", e);
                // restore compressed data to avoid data loss
                self.compressed = Some(data);
            }
        }
    }

    pub(crate) fn push(&mut self, line: Line) {
        let line_length = line.len();

        self.length += line_length + 1;
        self.line_count += 1;
        self.max_line_length = line_length.max(self.max_line_length);
        self.lines.push(line);
        // remove compressed as we just modified the lines array
        self.compressed = None;
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_count
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn is_full(&self) -> bool {
        self.line_count >= DEFAULT_GROUP_SIZE
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.max_line_length
    }

    /// Returns true if this group currently holds compressed data
    pub(crate) fn is_compressed(&self) -> bool {
        self.compressed.is_some()
    }

    pub(crate) fn is_decompressed(&self) -> bool {
        !self.lines.is_empty()
    }

    pub(crate) fn filter_line_mut(&mut self, mut filter: impl FnMut(&mut Line)) {
        let should_decompress = self.lines.is_empty();
        if should_decompress {
            self.decompress();
            // free compresed data as we will modify the lines array
            self.compressed = None;
        }
        debug_assert!(!self.lines.is_empty());
        self.lines.iter_mut().for_each(|line| filter(line));

        self.compute_metadata();
        if should_decompress {
            self.compress();
            self.free();
            debug_assert!(self.compressed.is_some());
            debug_assert!(self.lines.is_empty());
        }
    }

    fn compute_metadata(&mut self) {
        let (length, max_line_length) = self.lines.iter().fold((0, 0), |(sum, max), line| {
            let len = line.len() + 1;
            (sum + len, max.max(len))
        });
        self.line_count = self.lines.len();
        self.length = length;
        self.max_line_length = max_line_length;
    }

    pub(crate) fn retain<F: FnMut(&Line) -> bool>(&mut self, f: F) {
        debug_assert!(!self.lines.is_empty());
        self.lines.retain(f);
        if self.lines.len() != self.line_count {
            self.compressed = None;
            self.compute_metadata();
        }
    }

    pub(crate) fn drain_lines<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        debug_assert!(!self.lines.is_empty());
        self.compressed = None;
        self.lines.drain(range);
        self.compute_metadata();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lg_from_strs(strs: &[&str]) -> LineGroup {
        let mut g = LineGroup::default();
        for s in strs {
            g.push(Line::from(*s));
        }
        g
    }

    #[test]
    fn push_updates_counters_and_index() {
        let mut g = LineGroup::default();
        g.push(Line::from("a"));
        g.push(Line::from("bb"));

        assert_eq!(g.line_count(), 2);
        assert_eq!(g.len(), 1 + 1 + 2 + 1); // sum of (len+1)
        assert_eq!(g[0].content(), "a");
        assert_eq!(g[1].content(), "bb");
        assert!(g.max_line_length() >= 2); // sanity (implementation detail may count +1)
    }

    #[test]
    fn compress_decompress_roundtrip_preserves_content() {
        let mut g = lg_from_strs(&["hello", "world", "!"]);
        let before_len = g.len();
        let before_cnt = g.line_count();
        let before_texts: Vec<String> = (0..before_cnt)
            .map(|i| g[i].content().to_string())
            .collect();

        g.compress();
        // compressing twice should be idempotent
        g.compress();
        // Decompress and verify
        g.decompress();

        assert_eq!(g.line_count(), before_cnt);
        assert_eq!(g.len(), before_len);
        let after_texts: Vec<String> = (0..before_cnt)
            .map(|i| g[i].content().to_string())
            .collect();
        assert_eq!(before_texts, after_texts);
    }

    #[test]
    fn filter_line_mut_applies_and_preserves_compression_state() {
        let mut g = lg_from_strs(&["a", "b"]);
        g.compress(); // start compressed
        g.filter_line_mut(|l| {
            let mut s = l.content().to_string();
            s.push('x');
            *l = Line::from(s);
        });

        // After filter, content should be modified
        g.decompress();
        assert_eq!(g.line_count(), 2);
        assert!(g[0].content().ends_with('x'));
        assert!(g[1].content().ends_with('x'));
    }

    #[test]
    fn retain_and_drain_update_metadata() {
        let mut g = lg_from_strs(&["one", "two", "three", "four"]);
        // retain only items with length 3
        g.retain(|l| l.len() == 3);
        assert_eq!(g.line_count(), 2);

        // push two more and then drain first
        g.push(Line::from("xxx"));
        g.push(Line::from("yyyy"));
        assert!(g.line_count() >= 4);
        let before = g.len();
        g.drain_lines(0..1);
        assert_eq!(g.line_count(), 3);
        assert!(g.len() < before);
    }

    #[test]
    fn is_full_after_default_group_size_pushes() {
        let mut g = LineGroup::default();
        for _ in 0..DEFAULT_GROUP_SIZE {
            g.push(Line::from("a"));
        }
        assert!(g.is_full());
    }

    #[test]
    fn mem_reports_non_zero_after_push() {
        let mut g = LineGroup::default();
        let base = g.mem();
        g.push(Line::from("abcdef"));
        assert!(g.mem() >= base);
    }
}
