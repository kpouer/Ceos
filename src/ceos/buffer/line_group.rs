use crate::ceos::buffer::line::Line;
use std::ops::RangeBounds;
use std::ops::Index;
use log::warn;
use lz4::block;

pub(crate) const DEFAULT_GROUP_SIZE: usize = 1000;

#[derive(Debug)]
pub(crate) struct LineGroup {
    lines: Vec<Line>,
    compressed: Option<Vec<u8>>,
}

impl Default for LineGroup {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_GROUP_SIZE)
    }
}

impl LineGroup {
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self { lines: Vec::with_capacity(cap), compressed: None }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.lines.len() >= DEFAULT_GROUP_SIZE
    }

    pub(crate) fn compress(&mut self) {
        if self.lines.is_empty() {
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
                self.lines.clear();
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
                            self.lines.clear();
                        } else {
                            self.lines = text.split('\n').map(Line::from).collect();
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
        self.lines.push(line);
        self.compressed = None;
    }

    pub(crate) fn line_count(&self) -> usize { self.lines.len() }

    pub(crate) fn len(&self) -> usize {
        let text_size:usize = self.lines.iter().map(|l| l.len()).sum();
        let separators = self.lines.len().saturating_sub(1);
        text_size + separators
    }

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
