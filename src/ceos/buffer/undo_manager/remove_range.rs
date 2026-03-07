use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct RemoveRange {
    line: usize,
    offset: usize,
    text: String,
}

impl RemoveRange {
    pub(crate) const fn new(line: usize, offset: usize, text: String) -> Self {
        Self { line, offset, text }
    }
}

impl Edit for RemoveRange {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.insert_str(self.line, self.offset, &self.text);
        Position {
            line: self.line,
            column: self.offset,
        }
    }

    fn redo(&self, buffer: &mut Buffer) -> Position {
        buffer.delete_line_range(self.line, self.offset, self.text.len());
        Position {
            line: self.line,
            column: self.offset,
        }
    }
}
