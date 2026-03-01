use crate::ceos::search::SearchMatcher;
use std::borrow::Cow;

#[derive(Debug)]
pub struct SimpleSearchMatcher<'a> {
    query: Cow<'a, str>,
    case_insensitive: bool,
    whole_words: bool,
}

impl<'a> SimpleSearchMatcher<'a> {
    pub fn new(query: &'a str, case_sensitive: bool, whole_words: bool) -> Self {
        let query_text = if case_sensitive {
            Cow::Borrowed(query)
        } else {
            Cow::Owned(query.to_lowercase())
        };
        Self {
            query: query_text,
            case_insensitive: !case_sensitive,
            whole_words,
        }
    }
}

impl SearchMatcher for SimpleSearchMatcher<'_> {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        let search_slice = &line_text[from_col..];
        let search_text = if self.case_insensitive {
            search_slice.to_lowercase()
        } else {
            search_slice.to_string()
        };

        search_text.find(self.query.as_ref()).and_then(|found_idx| {
            let actual_idx = from_col + found_idx;
            let match_len = self.query.len();
            if self.whole_words {
                let before = if actual_idx == 0 {
                    ' '
                } else {
                    line_text.chars().nth(actual_idx - 1).unwrap_or(' ')
                };
                let after = line_text.chars().nth(actual_idx + match_len).unwrap_or(' ');
                if (before.is_alphanumeric() || before == '_')
                    || (after.is_alphanumeric() || after == '_')
                {
                    return None;
                }
            }
            Some((actual_idx, actual_idx + match_len))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_case_sensitive_exact_match() {
        let matcher = SimpleSearchMatcher::new("hello", true, false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_case_insensitive_match() {
        let matcher = SimpleSearchMatcher::new("hello", false, false);
        let result = matcher.search("Hello World", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_whole_words_match_found() {
        let matcher = SimpleSearchMatcher::new("word", true, true);
        let result = matcher.search("hello word world", 0);
        assert_eq!(result, Some((6, 10)));
    }

    #[test]
    fn test_search_whole_words_no_match_part_of_word() {
        let matcher = SimpleSearchMatcher::new("word", true, true);
        let result = matcher.search("hello wording world", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_from_column_position() {
        let matcher = SimpleSearchMatcher::new("world", true, false);
        let result = matcher.search("hello world world", 7);
        assert_eq!(result, Some((12, 17)));
    }

    #[test]
    fn test_search_no_match_found() {
        let matcher = SimpleSearchMatcher::new("missing", true, false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_match_at_beginning() {
        let matcher = SimpleSearchMatcher::new("hello", true, false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_match_at_end() {
        let matcher = SimpleSearchMatcher::new("world", true, false);
        let result = matcher.search("hello world", 0);
        assert_eq!(result, Some((6, 11)));
    }

    #[test]
    fn test_search_multiple_occurrences_finds_first() {
        let matcher = SimpleSearchMatcher::new("the", true, false);
        let result = matcher.search("the quick the brown", 0);
        assert_eq!(result, Some((0, 3)));
    }

    #[test]
    fn test_search_whole_words_at_boundaries() {
        let matcher = SimpleSearchMatcher::new("test", true, true);
        let result = matcher.search("test", 0);
        assert_eq!(result, Some((0, 4)));
    }

    #[test]
    fn test_search_case_insensitive_whole_words() {
        let matcher = SimpleSearchMatcher::new("word", false, true);
        let result = matcher.search("Hello WORD world", 0);
        assert_eq!(result, Some((6, 10)));
    }

    #[test]
    fn test_search_whole_words_with_punctuation() {
        let matcher = SimpleSearchMatcher::new("hello", true, true);
        let result = matcher.search("hello, world", 0);
        assert_eq!(result, Some((0, 5)));
    }
}
