use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::undo_manager::edit::Edit;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Replace {
    line: usize,
    start: usize,
    end: usize,
    text: String,
}

impl Replace {
    pub(crate) const fn new(line: usize, start: usize, end: usize, text: String) -> Self {
        Self {
            line,
            start,
            end,
            text,
        }
    }
}

impl Edit for Replace {
    fn undo(&self, _buffer: &mut Buffer) -> Position {
        // To undo a replacement, we would need the original text.
        // Assuming Replace here means we replaced [start, end) with `text`.
        // If we don't have the original text, we can't fully undo it.
        // But looking at Buffer, it seems Replace is not yet used for pushing to UndoManager.
        Position::default()
    }

    fn redo(&self, _buffer: &mut Buffer) -> Position {
        todo!()
    }
}
