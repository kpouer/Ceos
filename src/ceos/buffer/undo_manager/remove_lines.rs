use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::line::Line;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct RemoveLines {
    line: usize,
    lines: Vec<String>,
}

impl RemoveLines {
    pub(crate) fn new(line: usize, lines: Vec<Line>) -> Self {
        let lines = lines.into_iter().map(|line| line.into_content()).collect();
        Self { line, lines }
    }
}

impl Edit for RemoveLines {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.insert_lines(self.line, self.lines.clone());
        Position {
            line: self.line,
            column: 0,
        }
    }

    fn redo(&self, buffer: &mut Buffer) -> Position {
        buffer.drain_line_mut(self.line..self.line + self.lines.len());
        Position {
            line: self.line,
            column: 0,
        }
    }
}
