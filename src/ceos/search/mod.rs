pub mod regex_search_matcher;
pub mod simple_search_matcher;

pub trait SearchMatcher {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)>;
}
