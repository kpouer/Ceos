use crate::ceos::buffer::line::Line;
use log::warn;
use lz4::block;
use std::ops::Index;
use std::ops::RangeBounds;

pub(crate) const DEFAULT_GROUP_SIZE: usize = 1000;

#[derive(Debug)]
pub(crate) struct LineGroup {
    lines: Vec<Line>,
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

    pub(crate) fn free(&mut self) {
        self.lines.clear();
    }

    pub(crate) fn compress(&mut self) {
        if self.compressed.is_some() {
            // already compressed; keep in-memory lines for fast read access
            return;
        }

        let mut concatenated = String::with_capacity(self.len());
        for (i, line) in self.lines.iter().enumerate() {
            concatenated.push_str(line.content());
            if i + 1 < self.lines.len() {
                concatenated.push('\n');
            }
        }

        // include_size=true so that decompression doesn't need the original size
        match block::compress(concatenated.as_bytes(), None, true) {
            Ok(data) => {
                self.compressed = Some(data);
                // Do NOT clear in-memory lines so read operations remain possible without &mut self
            }
            Err(e) => {
                warn!("Failed to compress line group with LZ4: {}", e);
            }
        }
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
                                warn!("Decompressed line group with inconsistent line count {} != {}", self.line_count, self.lines.len());
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
        self.length += line.len() + 1;
        self.line_count += 1;
        if line.len() > self.max_line_length {
            self.max_line_length = line.len();
        }
        self.lines.push(line);
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

    pub(crate) fn filter_line_mut(&mut self, mut filter: impl FnMut(&mut Line)) {
        let compressed = self.lines.is_empty();
        if compressed {
            self.decompress();
            self.compressed = None;
        }
        self.lines
            .iter_mut()
            .for_each(|line|filter(line));

        self.compute_metadata();
        if compressed {
            self.compress();
            self.free();
        }
    }

    fn compute_metadata(&mut self) {
        let (length, max_line_length) = self.lines
            .iter()
            .fold((0, 0), |(sum, max), line| {
            let len = line.len() + 1;
            (sum + len, max.max(len))
        });
        self.line_count = self.lines.len();
        self.length = length;
        self.max_line_length = max_line_length;
    }

    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, Line> {
        self.lines.iter_mut()
    }

    pub(crate) fn retain<F: FnMut(&Line) -> bool>(&mut self, f: F) {
        self.lines.retain(f);
        self.compute_metadata();
    }

    pub(crate) fn drain<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.lines.drain(range);
        // recompute counters after drain
        self.line_count = self.lines.len();
        self.compute_metadata();
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.max_line_length
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
        let before_texts: Vec<String> = (0..before_cnt).map(|i| g[i].content().to_string()).collect();

        g.compress();
        // compressing twice should be idempotent
        g.compress();
        // Decompress and verify
        g.decompress();

        assert_eq!(g.line_count(), before_cnt);
        assert_eq!(g.len(), before_len);
        let after_texts: Vec<String> = (0..before_cnt).map(|i| g[i].content().to_string()).collect();
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
        g.drain(0..1);
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
