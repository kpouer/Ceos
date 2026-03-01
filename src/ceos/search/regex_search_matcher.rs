use crate::ceos::search::SearchMatcher;
use regex::{Error, Regex, RegexBuilder};

#[derive(Debug)]
pub(crate) struct RegexSearchMatcher {
    regex: Regex,
    whole_words: bool,
}

impl RegexSearchMatcher {
    pub(crate) fn new(
        pattern: &str,
        case_sensitive: bool,
        whole_words: bool,
    ) -> Result<Self, Error> {
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(!case_sensitive);
        let regex = builder.build()?;
        Ok(Self { regex, whole_words })
    }
}

impl SearchMatcher for RegexSearchMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        self.regex.find_at(line_text, from_col).and_then(|m| {
            let start = m.start();
            let end = m.end();
            if self.whole_words {
                let before = line_text[..start].chars().last().unwrap_or(' ');
                let after = line_text[end..].chars().next().unwrap_or(' ');
                if (before.is_alphanumeric() || before == '_')
                    || (after.is_alphanumeric() || after == '_')
                {
                    return None;
                }
            }
            Some((start, end))
        })
    }
}
