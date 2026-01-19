#[derive(Debug)]
pub(crate) struct TextTool<'a> {
    text: &'a str,
}

impl<'a> TextTool<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text }
    }

    // ... existing code ...
    pub(crate) fn find_word_start(&self, pos: usize) -> usize {
        if pos == 0 || self.text.is_empty() {
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

    fn is_word_separator(c: &char) -> bool {
        !c.is_alphanumeric() && *c != '_'
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
}

