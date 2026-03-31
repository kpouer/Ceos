use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::caret_possition::CaretPosition;
use std::fmt::Debug;

pub(crate) trait Edit: Debug + Send + Sync {
    fn undo(&self, buffer: &mut Buffer) -> CaretPosition;
    fn redo(&self, buffer: &mut Buffer) -> CaretPosition;
}
