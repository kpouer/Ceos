use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct InsertNewLine {
    line: usize,
}

impl InsertNewLine {
    pub(crate) const fn new(line: usize) -> Self {
        Self { line }
    }
}

impl Edit for InsertNewLine {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.
        panic!();
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
