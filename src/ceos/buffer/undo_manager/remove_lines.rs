use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct RemoveLines {
    lines: Vec<String>,
    line: usize,
}

impl RemoveLines {
    pub(crate) const fn new(lines: Vec<String>, line: usize) -> Self {
        Self { lines, line }
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

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
