pub(crate) mod regex_search_matcher;
pub(crate) mod simple_search_matcher;

pub(crate) trait SearchMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)>;
}
