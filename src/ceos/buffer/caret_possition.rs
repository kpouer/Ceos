use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::selection::Selection;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CaretPosition {
    Selection(Selection),
    Position(Position),
}
