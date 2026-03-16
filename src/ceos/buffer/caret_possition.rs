use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::selection::Selection;

#[derive(Debug)]
pub(crate) enum CaretPosition {
    Selection(Selection),
    Position(Position),
}
