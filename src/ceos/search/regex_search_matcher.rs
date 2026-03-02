use crate::ceos::search::SearchMatcher;
use crate::ceos::tools::text_tool::TextTool;
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
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()?;
        Ok(Self { regex, whole_words })
    }
}

impl SearchMatcher for RegexSearchMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)> {
        self.regex.find_at(line_text, from_col).and_then(|m| {
            let start = m.start();
            let end = m.end();

            if self.whole_words && !TextTool::is_whole_word(line_text, start, end) {
                return None;
            }

            Some((start, end))
        })
    }
}
