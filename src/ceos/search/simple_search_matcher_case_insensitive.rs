use crate::ceos::search::SearchMatcher;
use crate::ceos::tools::text_tool::TextTool;
use memchr::memchr2;

#[derive(Debug)]
pub struct SimpleSearchCaseInsensitiveMatcher {
    query: String,
    query_bytes: Option<Vec<u8>>,
    whole_words: bool,
}

impl SimpleSearchCaseInsensitiveMatcher {
    #[inline]
    pub fn new(query: &str, whole_words: bool) -> Self {
        let query_text = query.to_lowercase();
        let query_bytes = if query.is_ascii() {
            Some(query.as_bytes().to_vec())
        } else {
            None
        };
        Self {
            query: query_text,
            query_bytes,
            whole_words,
        }
    }

    #[inline]
    fn find_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return Some(0);
        }
        if needle.len() > haystack.len() {
            return None;
        }

        let n0_lo = needle[0].to_ascii_lowercase();
        let n0_hi = needle[0].to_ascii_uppercase();

        let mut start = 0;
        while start + needle.len() <= haystack.len() {
            let rel = memchr2(n0_lo, n0_hi, &haystack[start..])?;
            let pos = start + rel;

            if pos + needle.len() <= haystack.len() {
                let mut ok = true;
                for i in 0..needle.len() {
                    if !haystack[pos + i].eq_ignore_ascii_case(&needle[i]) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    return Some(pos);
                }
            }

            start = pos + 1;
        }

        None
    }
}

impl SearchMatcher for SimpleSearchCaseInsensitiveMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        let slice_from_col = &line_text[from_col..];

        let found_in_slice = if slice_from_col.is_ascii()
            && let Some(bytes) = &self.query_bytes
        {
            Self::find_ascii_case_insensitive(slice_from_col.as_bytes(), bytes)
        } else {
            slice_from_col.to_lowercase().find(&self.query)
        }?;

        let start = from_col + found_in_slice;
        let match_len_bytes = self.query.len();
        let end = start + match_len_bytes;

        if self.whole_words && !TextTool::is_whole_word(line_text, start, end) {
            return None;
        }

        Some((start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_case_insensitive_match() {
        let matcher = SimpleSearchCaseInsensitiveMatcher::new("hello", false);
        let result = matcher.search("Hello World", 0);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_search_case_insensitive_whole_words() {
        let matcher = SimpleSearchCaseInsensitiveMatcher::new("word", true);
        let result = matcher.search("Hello WORD world", 0);
        assert_eq!(result, Some((6, 10)));
    }
}
