use crate::ceos::gui::textpane::selection::Selection;
use buffer_core::position::Position;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CaretState {
    Selection(Selection),
    Position(Position),
}
