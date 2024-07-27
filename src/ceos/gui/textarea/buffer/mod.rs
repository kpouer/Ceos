use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::ceos::gui::textarea::buffer::line::Line;

pub(crate) mod line;

pub(crate) struct Buffer {
    path: String,
    content: Vec<Line>,
    length: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::from("")
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        let lines_iterator = text.lines();
        let mut content = Vec::with_capacity(lines_iterator.size_hint().0);
        lines_iterator.into_iter().for_each(|line| {
            content.push(Line::from(line));
        });

        let mut buffer = Self {
            path: String::new(),
            content,
            length: 0,
        };
        buffer.compute_length();
        buffer
    }
}

impl Buffer {
    pub(crate) fn new_from_file(path: String) -> anyhow::Result<Self> {
        let lines = read_lines(&path).unwrap();
        let mut content = Vec::new();
        let mut length = 0;
        #[allow(clippy::manual_flatten)]
        for line in lines {
            if let Ok(text) = line {
                length += text.len();
                content.push(Line::from(text));
            }
        }

        Ok(Self {
            path,
            content,
            length,
        })
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn line_text(&self, line: usize) -> &str {
        self.content[line].content()
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

    pub fn content(&self) -> &Vec<Line> {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Vec<Line> {
        &mut self.content
    }
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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