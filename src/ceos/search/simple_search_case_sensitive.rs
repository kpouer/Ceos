use crate::ceos::search::SearchMatcher;
use memchr::memmem;

#[derive(Debug)]
pub struct SimpleSearchCaseSensitiveMatcher {
    len: usize,
    finder: memmem::Finder<'static>,
    whole_words: bool,
}

impl SimpleSearchCaseSensitiveMatcher {
    #[inline]
    pub fn new(query: &str, whole_words: bool) -> Self {
        let finder = memmem::Finder::new(query.as_bytes()).into_owned();
        Self {
            len: query.len(),
            finder,
            whole_words,
        }
    }

    #[inline]
    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

impl SearchMatcher for SimpleSearchCaseSensitiveMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        let slice_from_col = &line_text[from_col..];

        let found_in_slice = self.finder.find(slice_from_col.as_bytes())?;

        let start = from_col + found_in_slice;
        let end = start + self.len;

        if self.whole_words {
            let before = line_text[..start].chars().last().unwrap_or(' ');
            if Self::is_word_char(before) {
                return None;
            }
            let after = line_text[end..].chars().next().unwrap_or(' ');

            if Self::is_word_char(after) {
                return None;
            }
        }

        Some((start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_case_sensitive_exact_match() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("hello", false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_whole_words_match_found() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("word", true);
        let result = matcher.search("hello word world", 0);
        assert_eq!(result, Some((6, 10)));
    }

    #[test]
    fn test_search_whole_words_no_match_part_of_word() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("word", true);
        let result = matcher.search("hello wording world", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_from_column_position() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("world", false);
        let result = matcher.search("hello world world", 7);
        assert_eq!(result, Some((12, 17)));
    }

    #[test]
    fn test_search_no_match_found() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("missing", false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_match_at_beginning() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("hello", false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_match_at_end() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("world", false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((6, 11)));
    }

    #[test]
    fn test_search_multiple_occurrences_finds_first() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("the", false);
        let result = matcher.search("the quick the brown", 0);
        assert_eq!(result, Some((0, 3)));
    }

    #[test]
    fn test_search_whole_words_at_boundaries() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("test", true);
        let result = matcher.search("test", 0);
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn test_search_whole_words_with_punctuation() {
        let matcher = SimpleSearchCaseSensitiveMatcher::new("hello", true);
        let result = matcher.search("hello, world", 0);
        assert_eq!(result, Some((0, 5)));
    }
}
