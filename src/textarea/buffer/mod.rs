use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use anyhow::Error;

use crate::textarea::buffer::line::Line;

pub(crate) mod line;
pub(crate) mod line_status;

pub(crate) struct Buffer {
    path: String,
    content: Vec<Line>,
    length: usize,
    total_length: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        let text = "Welcome to Ceos";
        Self {
            path: String::new(),
            content: vec![text.into()],
            length: text.len(),
            total_length: text.len(),
        }
    }
}

impl TryFrom<String> for Buffer {
    type Error = Error;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        let lines = read_lines(&path).unwrap();
        let mut content = Vec::new();
        let mut length = 0;
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
            total_length: length,
        })
    }
}

impl Buffer {
    pub(crate) fn line_text(&self, line: usize) -> &str {
        self.content[line].content()
    }

    pub(crate) fn line_count(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }

    pub(crate) fn total_length(&self) -> usize {
        self.total_length
    }

    pub(crate) fn max_line_length(&self) -> usize {
        self.content
            .iter()
            .map(|line| line.content().len())
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn compute_total_length(&mut self) -> usize {
        self.total_length = self.content.iter().map(|line| line.content().len()).sum();
        self.total_length
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
