use crate::ceos::gui::textpane::position::Position;

#[derive(Debug)]
pub(crate) struct TextRange {
    pub(super) start: Position,
    pub(super) end: Position,
}

impl TextRange {
    pub(crate) const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub(crate) const fn line_count(&self) -> usize {
        self.end.line - self.start.line + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_range_is_empty() {
        let range = TextRange::new(Position::ZERO, Position::ZERO);
        assert!(range.is_empty());
    }
}
