use crate::position::Position;
use crate::selection::Selection;

#[derive(Debug, PartialEq, Eq)]
pub enum CaretState {
    Selection(Selection),
    Position(Position),
}
