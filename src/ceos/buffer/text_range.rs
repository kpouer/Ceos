#[derive(Debug)]
pub(crate) struct TextRange {
    pub(super) start_line: usize,
    pub(super) start_column: usize,
    pub(super) end_line: usize,
    pub(super) end_column: usize,
}

impl TextRange {
    pub(crate) fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self { start_line, start_column, end_line, end_column }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.start_line == self.end_line && self.start_column == self.end_column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_range_is_empty() {
        let range = TextRange::new(0, 0, 0, 0);
        assert!(range.is_empty());
    }
}