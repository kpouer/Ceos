use crate::ceos::buffer::line::Line;
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use log::info;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::RangeBounds;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

pub(crate) mod line;

#[derive(Default)]
pub(crate) struct Buffer {
    pub(crate) path: Option<PathBuf>,
    pub(crate) content: Vec<Line>,
    length: usize,
    pub(crate) dirty: bool,
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        let lines_iterator = text.lines();
        let mut content = Vec::with_capacity(lines_iterator.size_hint().0);
        lines_iterator.into_iter().for_each(|line| {
            content.push(Line::from(line));
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
    pub(crate) fn new_from_file(path: PathBuf, sender: &Sender<Event>) -> anyhow::Result<Self> {
        let file_size = std::fs::metadata(&path)?.len() as usize;
        sender.send(BufferLoadingStarted(path.clone(), file_size))?;
        let file = File::open(&path)?;
        let mut line_text = String::with_capacity(500);
        let mut buffer = io::BufReader::new(file);
        let mut content = Vec::with_capacity(file_size / 100);
        let mut length = 0;
        let mut start = Instant::now();
        loop {
            let bytes = buffer.read_line(&mut line_text)?;
            if bytes == 0 {
                break;
            }
            length += line_text.len();
            content.push(Line::from(line_text.as_str()));
            line_text.clear();
            if start.elapsed() > Duration::from_millis(50) {
                sender.send(BufferLoading(path.clone(), length, file_size))?;
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

    pub(crate) fn drain_line_mut<R>(&mut self, range: R) -> usize
    where
        R: RangeBounds<usize>,
    {
        self.content.drain(range);
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn filter_line_mut<OP>(&mut self, filter: OP) -> usize
    where
        OP: Fn(&mut Line) + Sync + Send,
    {
        #[cfg(feature = "parallel")]
        {
            info!("Filtering lines in parallel mode");
            self.content.par_iter_mut().for_each(filter);
        }
        #[cfg(not(feature = "parallel"))]
        {
            info!("Filtering lines in non parallel mode");
            self.content.iter_mut().for_each(filter);
        }
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn retain_line_mut<OP>(&mut self, filter: OP) -> usize
    where
        OP: Fn(&Line) -> bool + Sync + Send,
    {
        #[cfg(feature = "parallel")]
        {
            info!("retain_line_mut lines in parallel mode");
            let mut tmp: Vec<Line> = Vec::with_capacity(0);
            std::mem::swap(&mut self.content, &mut tmp);
            self.content = tmp.into_par_iter().filter(|line| filter(line)).collect();
        }
        #[cfg(not(feature = "parallel"))]
        {
            info!("retain_line_mut lines in sequential mode");
            self.content.retain(filter);
        }
        let new_length = self.compute_length();
        self.dirty = true;
        new_length
    }

    pub(crate) fn line_text(&self, line: usize) -> &str {
        &self.content[line].content
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_new_from_text() {
        let buffer = Buffer::from("Hello\nWorld 22\nHow are you");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.max_line_length(), 11);
    }
}
