use crate::ceos::search::SearchMatcher;
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct SimpleSearchMatcher<'a> {
    query: Cow<'a, str>,
    case_insensitive: bool,
    whole_words: bool,
}

impl<'a> SimpleSearchMatcher<'a> {
    pub(crate) fn new(query: &'a str, case_sensitive: bool, whole_words: bool) -> Self {
        let query_text = if case_sensitive {
            Cow::Borrowed(query)
        } else {
            Cow::Owned(query.to_lowercase())
        };
        Self {
            query: query_text,
            case_insensitive: case_sensitive,
            whole_words,
        }
    }
}

impl SearchMatcher for SimpleSearchMatcher<'_> {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        let search_text = if self.case_insensitive {
            line_text.to_lowercase()
        } else {
            line_text.to_string()
        };

        let search_slice = &search_text[from_col..];
        search_slice
            .find(self.query.as_ref())
            .and_then(|found_idx| {
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
