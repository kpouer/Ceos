use std::fmt::Debug;

pub mod regex_search_matcher;
pub mod simple_search_case_sensitive;
pub mod simple_search_matcher_case_insensitive;

pub trait SearchMatcher: Debug {
    fn search(&self, line_text: &str, from_col: usize) -> Option<(usize, usize)>;
}
