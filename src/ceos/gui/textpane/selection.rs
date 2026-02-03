use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct Selection {
    /// The start position of the selection.
    pub(crate) start: Position,
    /// The end position of the selection.
    pub(crate) end: Position,
}

impl From<&Selection> for TextRange {
    fn from(selection: &Selection) -> TextRange {
        TextRange::new(
            selection.start.line,
            selection.start.column,
            selection.end.line,
            selection.end.column,
        )
    }
}