use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use anyhow::Error;
use log::info;

use crate::textarea::buffer::line::Line;
use crate::textarea::buffer::line_status::LineStatus;

pub(crate) mod line;
pub(crate) mod line_status;

#[derive(Default)]
pub(crate) struct Buffer {
    path: String,
    content: Vec<Line>,
    length: usize,
    total_length: usize,
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
    pub(crate) fn length(&self) -> usize {
        self.length
    }

    pub(crate) fn total_length(&self) -> usize {
        self.total_length
    }

    pub(crate) fn trim_deleted_lines(&mut self) {
        info!("Trim deleted lines");
        self.content
            .retain(|line| line.status() == &LineStatus::Normal);
        self.compute_total_length();
    }

    pub(crate) fn compute_lengths(&mut self) {
        self.compute_length();
        self.compute_total_length();
    }

    pub(crate) fn compute_length(&mut self) {
        self.length = self
            .content
            .iter()
            .filter(|line| line.status() == &LineStatus::Normal)
            .map(|line| line.content().len())
            .sum();
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

    pub(crate) fn deleted_line(&mut self, length: usize) {
        self.length -= length;
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
