use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Insert {
    line: usize,
    start: usize,
    text: String,
}

impl Insert {
    pub(crate) const fn new(line: usize, start: usize, text: String) -> Self {
        Self { line, start, text }
    }
}

impl Edit for Insert {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.delete_line_range(self.line, self.start, self.text.len());
        Position {
            line: self.line,
            column: self.start,
        }
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
