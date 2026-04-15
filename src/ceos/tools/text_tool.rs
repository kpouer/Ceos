use std::ops::{Index, Range, RangeFrom, RangeTo};

#[derive(Debug)]
pub(crate) struct TextTool<'a> {
    text: &'a str,
}

impl<'a> TextTool<'a> {
    pub(crate) const fn new(text: &'a str) -> Self {
        Self { text }
    }

    // ... existing code ...
    pub(crate) fn find_word_start(&self, pos: usize) -> usize {
        if pos == 0 || self.text.is_empty() || pos >= self.text.len() {
            return 0;
        }

        self.text[..pos]
            .char_indices()
            .rev()
            .find(|(_, c)| Self::is_word_separator(c))
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
    }

    pub(crate) fn find_word_end(&self, pos: usize) -> usize {
        if self.text.is_empty() || pos >= self.text.len() {
            return self.text.len();
        }
        self.text
            .chars()
            .skip(pos)
            .position(|c| Self::is_word_separator(&c))
            .map(|i| pos + i)
            .unwrap_or(self.text.len())
    }

    #[inline]
    pub(crate) fn is_word_separator(c: &char) -> bool {
        !c.is_alphanumeric() && *c != '_'
    }

    #[inline]
    pub(crate) fn is_whole_word(line_text: &str, start: usize, end: usize) -> bool {
        let before = line_text[..start].chars().last().unwrap_or(' ');
        if !Self::is_word_separator(&before) {
            return false;
        }

        let after = line_text[end..].chars().next().unwrap_or(' ');

        if !Self::is_word_separator(&after) {
            return false;
        }

        true
    }

    pub(crate) fn char_to_byte_idx(&self, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(self.text.len())
    }
}

impl<'a> From<&'a str> for TextTool<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> AsRef<str> for TextTool<'a> {
    fn as_ref(&self) -> &str {
        self.text
    }
}

impl<'a> Index<Range<usize>> for TextTool<'a> {
    type Output = str;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        let start = self.char_to_byte_idx(index.start);
        let end = self.char_to_byte_idx(index.end);
        &self.text[start..end]
    }
}

impl<'a> Index<RangeFrom<usize>> for TextTool<'a> {
    type Output = str;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        let start = self.char_to_byte_idx(index.start);
        &self.text[start..]
    }
}

impl<'a> Index<RangeTo<usize>> for TextTool<'a> {
    type Output = str;

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        let end = self.char_to_byte_idx(index.end);
        &self.text[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_word_start_at_beginning() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_start(0), 0);
    }

    #[test]
    fn test_find_word_start_middle_of_word() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_start(3), 0);
        assert_eq!(tool.find_word_start(8), 6);
    }

    #[test]
    fn test_find_word_start_at_separator() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_start(5), 0);
        assert_eq!(tool.find_word_start(6), 6);
    }

    #[test]
    fn test_find_word_start_empty_text() {
        let tool = TextTool::new("");
        assert_eq!(tool.find_word_start(0), 0);
    }

    #[test]
    fn test_find_word_end_at_end() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_end(11), 11);
        assert_eq!(tool.find_word_end(20), 11);
    }

    #[test]
    fn test_find_word_end_middle_of_word() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_end(0), 5);
        assert_eq!(tool.find_word_end(2), 5);
        assert_eq!(tool.find_word_end(6), 11);
    }

    #[test]
    fn test_find_word_end_at_separator() {
        let tool = TextTool::new("hello world");
        assert_eq!(tool.find_word_end(5), 5);
    }

    #[test]
    fn test_find_word_end_empty_text() {
        let tool = TextTool::new("");
        assert_eq!(tool.find_word_end(0), 0);
    }

    #[test]
    fn test_word_boundaries_with_underscores() {
        let tool = TextTool::new("hello_world");
        assert_eq!(tool.find_word_start(6), 0);
        assert_eq!(tool.find_word_end(0), 11);
    }

    #[test]
    fn test_word_boundaries_with_multiple_separators() {
        let tool = TextTool::new("hello  world");
        assert_eq!(tool.find_word_start(8), 7);
        assert_eq!(tool.find_word_end(0), 5);
        assert_eq!(tool.find_word_end(5), 5);
        assert_eq!(tool.find_word_end(6), 6);
    }

    #[test]
    fn test_word_boundaries_with_punctuation() {
        let tool = TextTool::new("hello, world!");
        assert_eq!(tool.find_word_start(3), 0);
        assert_eq!(tool.find_word_end(0), 5);
        assert_eq!(tool.find_word_start(9), 7);
        assert_eq!(tool.find_word_end(7), 12);
    }

    #[test]
    fn test_is_whole_word_at_beginning() {
        let line = "hello world";
        assert!(TextTool::is_whole_word(line, 0, 5));
    }

    #[test]
    fn test_is_whole_word_at_end() {
        let line = "hello world";
        assert!(TextTool::is_whole_word(line, 6, 11));
    }

    #[test]
    fn test_is_whole_word_in_middle() {
        let line = "hello world today";
        assert!(TextTool::is_whole_word(line, 6, 11));
    }

    #[test]
    fn test_is_whole_word_with_punctuation() {
        let line = "hello, world!";
        assert!(TextTool::is_whole_word(line, 0, 5));
        assert!(TextTool::is_whole_word(line, 7, 12));
    }

    #[test]
    fn test_is_whole_word_partial_at_start() {
        let line = "hello world";
        assert!(!TextTool::is_whole_word(line, 0, 3));
    }

    #[test]
    fn test_is_whole_word_partial_at_end() {
        let line = "hello world";
        assert!(!TextTool::is_whole_word(line, 8, 11));
    }

    #[test]
    fn test_is_whole_word_partial_both_sides() {
        let line = "hello world";
        assert!(!TextTool::is_whole_word(line, 1, 4));
    }

    #[test]
    fn test_is_whole_word_with_underscores() {
        let line = "hello_world test";
        assert!(TextTool::is_whole_word(line, 0, 11));
        assert!(!TextTool::is_whole_word(line, 0, 5));
        assert!(!TextTool::is_whole_word(line, 6, 11));
    }

    #[test]
    fn test_is_whole_word_with_multiple_separators() {
        let line = "hello  world";
        assert!(TextTool::is_whole_word(line, 0, 5));
        assert!(TextTool::is_whole_word(line, 7, 12));
    }

    #[test]
    fn test_is_whole_word_entire_line() {
        let line = "hello";
        assert!(TextTool::is_whole_word(line, 0, 5));
    }
}
