use crate::position::Position;
use crate::text_range::TextRange;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Selection {
    /// The start position of the selection.
    pub start: Position,
    /// The end position of the selection.
    pub end: Position,
}

impl Selection {
    pub fn new(start: Position, end: Position) -> Self {
        debug_assert!(start < end);
        Self { start, end }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub const fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
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
