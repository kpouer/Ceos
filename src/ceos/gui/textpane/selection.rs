use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Selection {
    pub(crate) start: Position,
    pub(crate) end: Position,
}
