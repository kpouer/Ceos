use crate::ceos::search::SearchMatcher;

#[derive(Debug)]
pub(crate) struct SimpleSearchMatcher {
    query: String,
    case_insensitive: bool,
    whole_words: bool,
}

impl SimpleSearchMatcher {
    pub(crate) fn new(query: &str, case_insensitive: bool, whole_words: bool) -> Self {
        let query_text = if case_insensitive {
            query.to_lowercase()
        } else {
            query.to_string()
        };
        Self {
            query: query_text,
            case_insensitive,
            whole_words,
        }
    }
}

impl SearchMatcher for SimpleSearchMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        let search_text = if self.case_insensitive {
            line_text.to_lowercase()
        } else {
            line_text.to_string()
        };

        let search_slice = &search_text[from_col..];
        search_slice.find(&self.query).and_then(|found_idx| {
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
