use crate::position::Position;
use crate::text_range::Selection;

#[derive(Debug, PartialEq, Eq)]
pub enum CaretState {
    Selection(Selection),
    Position(Position),
}
