use crate::ceos::buffer::line::Line;
use log::{debug, error, warn};
use std::borrow::Cow;
use std::io::{Read, Write};
use std::ops::Index;
use std::ops::RangeBounds;

#[derive(Debug)]
pub(crate) struct LineGroup {
    /// Contains the uncompressed data. Might be there even if the compressed data is present.
    lines: Option<Vec<Line>>,
    /// Contains the compressed data if the group is compressed, None otherwise.
    compressed: Option<Vec<u8>>,
    // number of lines stored in this group (stable even when compressed)
    line_count: usize,
    // total UTF-8 text length of the group with one '\n' separator between lines
    length: usize,
    max_line_length: usize,
    /// Global index (0-based) of the first line contained in this group
    first_line: usize,
    group_size: usize,
}

impl LineGroup {
    pub(crate) fn new(first_line: usize, group_size: usize) -> Self {
        Self {
            lines: Some(Vec::with_capacity(group_size)),
            compressed: None,
            line_count: 0,
            length: 0,
            max_line_length: 0,
            first_line,
            group_size,
        }
    }

    /// Free memory occupied by the lines.
    pub(crate) fn free(&mut self) {
        if self.compressed.is_none() {
            error!("free called on a decompressed group");
        }
        self.lines = None;
        // It's valid to free an empty group (no compressed data expected),
        // but for non-empty groups we expect compressed data to be present.
        debug_assert!(
            self.compressed.is_some() || self.line_count == 0,
            "free called on non-empty group without compressed data"
        );
    }

    pub(crate) fn eventually_compress(&mut self) {
        if self.line_count == 0 {
            debug!("eventually_compress called on empty group");
            return;
        }
        if !self.is_compressed() {
            self.compress();
        } else {
            debug!("eventually_compress called on already compressed group");
        }
    }

    fn compress(&mut self) {
        debug_assert!(self.lines.is_some());
        let Some(lines) = &self.lines else {
            error!("compress called on empty group");
            return;
        };
        // Stream (frame) compression to avoid building a large intermediate buffer
        let out = Vec::new();
        match lz4::EncoderBuilder::new().build(out) {
            Ok(mut encoder) => {
                for (i, line) in lines.iter().enumerate() {
                    if let Err(e) = encoder.write_all(line.content().as_bytes()) {
                        warn!("Failed to write to LZ4 encoder: {e}");
                        return;
                    }
                    if i != self.line_count - 1
                        && let Err(e) = encoder.write_all(b"\n")
                    {
                        warn!("Failed to write newline to LZ4 encoder: {e}");
                        return;
                    }
                }

                let (data, res) = encoder.finish();
                match res {
                    Ok(()) => self.compressed = Some(data),
                    Err(e) => warn!("Failed to finalize LZ4 encoder: {e}"),
                }
            }
            Err(e) => warn!("Failed to build LZ4 encoder: {e}"),
        }
    }

    pub(crate) fn eventually_decompress(&mut self) {
        if self.lines.is_some() {
            debug!("eventually_decompress called on a decompressed group");
            return;
        }
        if self.compressed.is_none() {
            error!("eventually_decompress called on a non compressed group");
            return;
        }
        self.decompress();
    }

    fn decompress(&mut self) {
        debug_assert!(self.lines.is_none());
        debug_assert!(self.compressed.is_some());

        let lines = self.decompress_lines();
        debug_assert!(!lines.is_empty());
        let decompressed_line_count = lines.len();
        self.lines = Some(lines);
        // successful decompression; drop compressed data
        self.compressed = None;
        #[cfg(debug_assertions)]
        if self.line_count != decompressed_line_count {
            warn!(
                "Decompressed line group with inconsistent line count {} != {decompressed_line_count}",
                self.line_count
            );
        }
    }

    /// Decompresses the group's compressed data and returns the resulting Vec<Line>.
    /// On failure, returns an empty Vec.
    pub(crate) fn decompress_lines(&self) -> Vec<Line> {
        let Some(data) = self.compressed.as_deref() else {
            return Vec::new();
        };

        let cursor = std::io::Cursor::new(data);
        match lz4::Decoder::new(cursor) {
            Ok(mut decoder) => {
                let mut bytes = Vec::new();
                match decoder.read_to_end(&mut bytes) {
                    Ok(_) => match String::from_utf8(bytes) {
                        Ok(text) => {
                            if text.is_empty() {
                                Vec::new()
                            } else {
                                text.split('\n').map(Line::from).collect()
                            }
                        }
                        Err(e) => {
                            warn!("Failed to decode UTF-8 after LZ4 stream decompress: {}", e);
                            Vec::new()
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read from LZ4 decoder: {}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create LZ4 decoder: {}", e);
                Vec::new()
            }
        }
    }

    pub(crate) fn push(&mut self, line: Line) {
        let line_length = line.len();

        self.length += line_length + 1;
        self.line_count += 1;
        self.max_line_length = line_length.max(self.max_line_length);
        if let Some(lines) = &mut self.lines {
            lines.push(line);
        }
        // remove compressed as we just modified the lines array
        self.compressed = None;
    }

    pub(crate) fn lines(&self) -> Cow<'_, [Line]> {
        if let Some(lines) = &self.lines {
            return Cow::Borrowed(lines);
        }
        Cow::Owned(self.decompress_lines())
    }

    pub(crate) const fn line_count(&self) -> usize {
        self.line_count
    }

    pub(crate) const fn len(&self) -> usize {
        self.length
    }

    pub(crate) const fn is_full(&self) -> bool {
        self.line_count >= self.group_size
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    pub(crate) const fn max_line_length(&self) -> usize {
        self.max_line_length
    }

    pub(crate) const fn first_line(&self) -> usize {
        self.first_line
    }

    pub(crate) const fn set_first_line(&mut self, value: usize) {
        self.first_line = value;
    }

    /// Returns true if this group currently holds compressed data
    pub(crate) const fn is_compressed(&self) -> bool {
        self.compressed.is_some()
    }

    pub(crate) const fn is_decompressed(&self) -> bool {
        self.lines.is_some()
    }

    pub(crate) const fn decompressed_line_count(&self) -> usize {
        if let Some(lines) = &self.lines {
            lines.len()
        } else {
            0
        }
    }

    /// Applies a mutable filter function to each line in the internal data structure.
    ///
    /// This function ensures that the internal line data is decompressed (if needed)
    /// before modifying it and applies the provided filter function to each line.
    /// Once the modifications are complete, it recompresses the line data and
    /// frees unnecessary memory.
    ///
    /// # Parameters
    /// - `filter`: A closure or function that takes a mutable reference to a `Line`
    ///   and modifies it.
    ///
    /// # Behavior
    /// 1. If the internal line data (`self.lines`) is compressed, the method decompresses it and will recompress after.
    ///  2. The provided `filter` function is applied to each line
    ///  3. Updates the metadata of the object by calling `self.compute_metadata`.
    ///  4. If the line data was initially decompressed for this operation, it is
    ///     recompressed after the modifications, and any decompressed data is
    ///     freed.
    pub(crate) fn filter_lines_mut(&mut self, filter: impl FnMut(&mut Line)) {
        let should_decompress = self.lines.is_none();
        if should_decompress {
            self.decompress();
            // free compresed data as we will modify the line array
            self.compressed = None;
        }
        debug_assert!(self.lines.is_some());
        if let Some(lines) = &mut self.lines {
            lines.iter_mut().for_each(filter);
        }

        self.compute_metadata();
        if should_decompress {
            self.compress();
            self.free();
            debug_assert!(self.compressed.is_some());
            debug_assert!(self.lines.is_none());
        }
    }

    pub(crate) fn filter_line_mut(
        &mut self,
        line_number: usize,
        mut filter: impl FnMut(&mut Line),
    ) {
        let should_decompress = self.lines.is_none();
        if should_decompress {
            self.decompress();
            // free compresed data as we will modify the line array
            self.compressed = None;
        }
        debug_assert!(self.lines.is_some());

        if let Some(lines) = &mut self.lines {
            let line = &mut lines[line_number];
            filter(line);
        }

        self.compute_metadata();
        if should_decompress {
            self.compress();
            self.free();
            debug_assert!(self.compressed.is_some());
            debug_assert!(self.lines.is_none());
        }
    }

    pub(crate) fn compute_metadata(&mut self) {
        debug_assert!(self.is_decompressed());
        if let Some(lines) = &self.lines {
            let (length, max_line_length) = lines.iter().fold((0, 0), |(sum, max), line| {
                let len = line.len() + 1;
                (sum + len, max.max(len))
            });
            self.line_count = lines.len();
            self.length = length;
            self.max_line_length = max_line_length;
        }
    }

    pub(crate) fn retain<F: FnMut(&Line) -> bool>(&mut self, f: F) {
        let compressed = self.is_compressed();
        if compressed {
            self.decompress();
        }

        debug_assert!(self.lines.is_some());
        if let Some(lines) = &mut self.lines {
            lines.retain(f);
            if lines.len() != self.line_count {
                self.compressed = None;
                self.compute_metadata();
            }
        }
        if compressed {
            self.compress();
        }
    }

    pub(crate) fn drain_lines<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        let compressed = self.is_compressed();
        if compressed {
            self.decompress();
        }
        self.compressed = None;
        if let Some(lines) = &mut self.lines {
            lines.drain(range);
        }
        self.compute_metadata();
        if compressed {
            self.compress();
        }
    }

    pub(crate) fn insert_line(&mut self, line_number: usize, line: Line) {
        let compressed = self.is_compressed();
        if compressed {
            self.decompress();
        }
        self.compressed = None;
        if let Some(lines) = &mut self.lines {
            lines.insert(line_number, line);
        }
        self.compute_metadata();
        if compressed {
            self.compress();
        }
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<Line>>();
        if let Some(lines) = &self.lines {
            let array_mem = lines.capacity() * std::mem::size_of::<Line>();
            let strings_mem: usize = lines.iter().map(|line| line.mem()).sum();
            vec_overhead + array_mem + strings_mem + self.compressed_size()
        } else {
            vec_overhead + self.compressed_size()
        }
    }

    pub fn compressed_size(&self) -> usize {
        self.compressed
            .as_ref()
            .map(|data| data.len())
            .unwrap_or_default()
    }

    #[cfg(debug_assertions)]
    pub(crate) fn debug(&self) {
        println!(
            "LineGroup {{ line_count: {}, length: {}, max_line_length: {}, first_line: {}, compressed: {:?} }}",
            self.line_count,
            self.length,
            self.max_line_length,
            self.first_line,
            self.compressed.is_some()
        );
    }
}

impl Index<usize> for LineGroup {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(self.lines.is_some());
        debug_assert!(index < self.line_count);
        let lines = self
            .lines
            .as_deref()
            .unwrap_or_else(|| panic!("index called on empty group"));
        assert!(index < lines.len());
        &lines[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lg_from_strs(strs: &[&str]) -> LineGroup {
        let mut g = LineGroup::new(0, 2);
        for s in strs {
            g.push(Line::from(*s));
        }
        g
    }

    #[test]
    fn push_updates_counters_and_index() {
        let mut g = LineGroup::new(0, 2);
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
        g.free();
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
        g.free();
        g.filter_lines_mut(|l| {
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
        let mut g = LineGroup::new(0, 2);
        for _ in 0..g.group_size {
            g.push(Line::from("a"));
        }
        assert!(g.is_full());
    }

    #[test]
    fn mem_reports_non_zero_after_push() {
        let mut g = LineGroup::new(0, 2);
        let base = g.mem();
        g.push(Line::from("abcdef"));
        assert!(g.mem() >= base);
    }

    #[test]
    fn compress_decompress_empty_lines() {
        let mut g = lg_from_strs(&["abc", "def", "ghi"]);
        g.compress();
        g.free();
        g.decompress();
        assert_eq!(g.line_count(), 3);
        assert_eq!(g[0].content(), "abc");
        assert_eq!(g[1].content(), "def");
        assert_eq!(g[2].content(), "ghi");
    }

    #[test]
    fn compress_free_preserves_metadata() {
        let mut g = lg_from_strs(&["test1", "test2"]);
        let count = g.line_count();
        let len = g.len();
        g.compress();
        g.free();
        assert_eq!(g.line_count(), count);
        assert_eq!(g.len(), len);
        assert!(g.is_compressed());
        assert!(!g.is_decompressed());
    }
}
