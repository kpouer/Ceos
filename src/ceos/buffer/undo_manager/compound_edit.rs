use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct CompoundEdit {
    edits: Vec<Box<dyn Edit>>,
}

impl CompoundEdit {
    pub(crate) fn new(edits: Vec<Box<dyn Edit>>) -> Self {
        Self { edits }
    }
}

impl Edit for CompoundEdit {
    fn undo(&self, buffer: &mut Buffer) -> Position {
        let mut pos = Position::default();
        for e in self.edits.iter().rev() {
            pos = e.undo(buffer);
        }
        pos
    }

    fn redo(&self, buffer: &mut Buffer) -> Position {
        let mut pos = Position::default();
        for e in self.edits.iter() {
            pos = e.redo(buffer);
        }
        pos
    }
}
