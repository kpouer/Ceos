use crate::position::Position;
use log::error;

pub type Selection = TextRange;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextRange {
    pub start: Position,
    pub end: Position,
}

impl TextRange {
    pub fn new(start: Position, end: Position) -> Self {
        if end < start {
            error!("New TextRange start must be < end : {start}<{end}");
            Self {
                start: end,
                end: start,
            }
        } else {
            Self { start, end }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub const fn line_count(&self) -> usize {
        self.end.line - self.start.line + 1
    }

    #[inline]
    pub const fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
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
