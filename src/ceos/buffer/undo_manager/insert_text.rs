use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct InsertText {
    line: usize,
    offset: usize,
    length: usize,
}

impl InsertText {
    pub(crate) fn new(line: usize, offset: usize, length: usize) -> Self {
        Self {
            line,
            offset,
            length,
        }
    }
}

impl Edit for InsertText {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.delete_line_range(self.line, self.offset, self.length);
        Position {
            line: self.line,
            column: self.offset,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
