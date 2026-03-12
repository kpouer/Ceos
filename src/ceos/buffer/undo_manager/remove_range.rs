use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct RemoveRange {
    position: Position,
    text: String,
}

impl RemoveRange {
    pub(crate) const fn new(position: Position, text: String) -> Self {
        Self { position, text }
    }
}

impl Edit for RemoveRange {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        buffer.insert_str(self.position, &self.text);
        self.position
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
