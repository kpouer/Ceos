use crate::ceos::buffer::line::Line;
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::ops::{Index, RangeBounds};
use std::path::PathBuf;
use std::slice::Iter;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Default, Debug)]
pub(crate) struct Buffer {
    pub(crate) path: Option<PathBuf>,
    content: Vec<Line>,
    length: usize,
    pub(crate) dirty: bool,
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        let lines_iterator = text.lines();
        let mut content = Vec::with_capacity(lines_iterator.size_hint().0);
        lines_iterator
            .into_iter()
            .map(Line::from)
            .for_each(|line| {
            content.push(line);
        });

        let mut buffer = Self {
            path: None,
            content,
            length: 0,
            dirty: false,
        };
        buffer.compute_length();
        buffer
    }
}

impl Buffer {
    pub(crate) fn new_from_file(path: PathBuf, sender: &Sender<Event>) -> Result<Self, std::io::Error> {
        let file_size = std::fs::metadata(&path)?.len() as usize;
        let _ = sender.send(BufferLoadingStarted(path.clone(), file_size));
        let file = File::open(&path)?;
        let buffer = io::BufReader::new(file);
        let mut content = Vec::with_capacity(file_size / 100);
        let mut length = 0;
        let mut start = Instant::now();
        for line_text in buffer.lines().flatten() {
            // keep same semantics as previous loop: add only the line content length
            length += line_text.len();
            content.push(Line::from(line_text.as_str()));
            if start.elapsed() > Duration::from_millis(50) {
                let _ = sender.send(BufferLoading(path.clone(), length, file_size));
                start = Instant::now();
            }
        }

        Ok(Self {
            path: Some(path),
            content,
            length,
            dirty: false,
        })
    }

    pub(crate) fn iter(&self) -> Iter<'_, Line> {
        self.content.iter()
    }

    pub(crate) fn drain_line_mut<R>(&mut self, range: R) -> usize
    where
        R: RangeBounds<usize>,
    {
        self.content.drain(range);
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn filter_line_mut(&mut self, filter: impl FnMut(&mut Line)) -> usize {
        self.content.iter_mut().for_each(filter);
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn retain_line_mut(&mut self, filter: impl FnMut(&Line) -> bool) -> usize {
        self.content.retain(filter);
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn line_text(&self, line: usize) -> &str {
        &self.content[line].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.content
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn compute_length(&mut self) -> usize {
        self.length = self.content.iter().map(|line| line.len()).sum();
        self.length += self.line_count();
        self.length
    }

    pub(crate) fn mem(&self) -> usize {
        let vec_overhead = std::mem::size_of::<Vec<Line>>();
        let array_mem = self.content.capacity() * std::mem::size_of::<Line>();
        let strings_mem: usize = self
            .content
            .iter()
            .map(|line| line.mem())
            .sum();
        vec_overhead + array_mem + strings_mem
    }
}

impl Index<usize> for Buffer {
    type Output = Line;

    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::ceos::buffer::buffer::Buffer;
    #[test]
    fn test_buffer_new_from_text() {
        let buffer = Buffer::from("Hello\nWorld 22\nHow are you");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.max_line_length(), 11);
    }
}