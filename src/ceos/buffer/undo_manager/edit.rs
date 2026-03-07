use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::position::Position;
use std::fmt::Debug;

pub(crate) trait Edit: Debug + Send + Sync {
    fn undo(&self, buffer: &mut Buffer) -> Position;
    fn redo(&self, buffer: &mut Buffer) -> Position;
}
