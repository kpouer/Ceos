use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::position::Position;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) struct Selection {
    /// The start position of the selection.
    pub(crate) start: Position,
    /// The end position of the selection.
    pub(crate) end: Position,
}

impl Selection {
    pub(crate) fn new(start: Position, end: Position) -> Self {
        debug_assert!(start < end);
        Self { start, end }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl From<&Selection> for TextRange {
    fn from(selection: &Selection) -> TextRange {
        TextRange::new(selection.start, selection.end)
    }
}

impl From<TextRange> for Selection {
    fn from(text_range: TextRange) -> Self {
        Self {
            start: text_range.start,
            end: text_range.end,
        }
    }
}
