use crate::position::Position;

#[derive(Debug, Copy, Clone)]
pub struct TextRange {
    pub start: Position,
    pub end: Position,
}

impl TextRange {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub const fn line_count(&self) -> usize {
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
