use std::fmt::{Display, Formatter};
use std::ops::{Index, RangeBounds};
use std::slice::SliceIndex;
use std::string::Drain;

#[derive(Default, Debug, Clone)]
pub struct Line {
    content: String,
}

impl<T: Into<String>> From<T> for Line {
    fn from(content: T) -> Self {
        let mut content = content.into();
        content.shrink_to_fit();
        Self { content }
    }
}

impl Line {
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.content.len()
    }

    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[inline]
    pub fn into_content(self) -> String {
        self.content
    }

    #[inline]
    pub(crate) const fn mem(&self) -> usize {
        self.content.capacity()
    }

    /// Removes and returns a specified range of characters from the `content` field of the struct.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_>
    where
        R: RangeBounds<usize>,
    {
        self.content.drain(range)
    }

    #[inline]
    pub fn push_str(&mut self, str: &str) {
        self.content.push_str(str);
    }

    #[inline]
    pub fn insert_str(&mut self, idx: usize, str: &str) {
        self.content.insert_str(idx, str);
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        let byte_idx = self
            .content
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len());
        self.content.insert(byte_idx, ch);
    }
}

impl<I: SliceIndex<str>> Index<I> for Line {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.content[index]
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::Line;

    #[test]
    fn default_line_is_empty() {
        let line = Line::default();
        assert!(line.is_empty());
        assert_eq!(line.len(), 0);
        assert_eq!(line.content(), "");
        assert!(line.mem() >= line.len());
    }

    #[test]
    fn from_str_populates_content() {
        let line = Line::from("hello");
        assert!(!line.is_empty());
        assert_eq!(line.len(), 5);
        assert_eq!(line.content(), "hello");
        assert!(line.mem() >= line.len());
    }

    #[test]
    fn drain_full_range_empts_line() {
        let mut line = Line::from("hello");
        line.drain(0..5);
        assert!(line.is_empty());
        assert_eq!(line.len(), 0);
        assert_eq!(line.content(), "");
    }

    #[test]
    fn drain_prefix() {
        let mut line = Line::from("hello");
        line.drain(0..2); // remove "he"
        assert_eq!(line.content(), "llo");
        assert_eq!(line.len(), 3);
    }

    #[test]
    fn drain_suffix() {
        let mut line = Line::from("hello");
        line.drain(3..); // remove from index 3 to end: remove "lo"
        assert_eq!(line.content(), "hel");
        assert_eq!(line.len(), 3);
    }

    #[test]
    fn drain_inclusive_middle() {
        let mut line = Line::from("abcdef");
        line.drain(1..=3); // remove b,c,d
        assert_eq!(line.content(), "aef");
        assert_eq!(line.len(), 3);
    }

    #[test]
    fn drain_empty_range_noop() {
        let mut line = Line::from("hello");
        line.drain(2..2); // no chars removed
        assert_eq!(line.content(), "hello");
        assert_eq!(line.len(), 5);
        assert_eq!(line.mem(), 5);
    }

    #[test]
    fn push_str_appends_content() {
        let mut line = Line::from("hello");
        line.push_str(" world");
        assert_eq!(line.content(), "hello world");
        assert_eq!(line.len(), 11);
        assert!(!line.is_empty());
    }

    #[test]
    fn indexing_returns_slice() {
        let line = Line::from("hello world");
        assert_eq!(&line[0..5], "hello");
        assert_eq!(&line[6..], "world");
        assert_eq!(&line[..], "hello world");
        assert_eq!(&line[0..=4], "hello");
    }

    #[test]
    fn insert() {
        let mut line = Line::from("he€llo");
        line.insert(3, 'a');
    }
}
