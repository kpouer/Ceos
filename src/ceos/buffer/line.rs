use std::ops::RangeBounds;

#[derive(Default, Debug, Clone)]
pub(crate) struct Line {
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
    pub(crate) fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
    pub(crate) fn len(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    pub(crate) fn mem(&self) -> usize {
        self.content.capacity()
    }

    pub(crate) fn drain<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.content.drain(range);
        self.content.shrink_to_fit();
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
        assert_eq!(line.mem(), 0);
    }

    #[test]
    fn drain_prefix() {
        let mut line = Line::from("hello");
        line.drain(0..2); // remove "he"
        assert_eq!(line.content(), "llo");
        assert_eq!(line.len(), 3);
        assert_eq!(line.mem(), 3);
    }

    #[test]
    fn drain_suffix() {
        let mut line = Line::from("hello");
        line.drain(3..); // remove from index 3 to end: remove "lo"
        assert_eq!(line.content(), "hel");
        assert_eq!(line.len(), 3);
        assert_eq!(line.mem(), 3);
    }

    #[test]
    fn drain_inclusive_middle() {
        let mut line = Line::from("abcdef");
        line.drain(1..=3); // remove b,c,d
        assert_eq!(line.content(), "aef");
        assert_eq!(line.len(), 3);
        assert_eq!(line.mem(), 3);
    }

    #[test]
    fn drain_empty_range_noop() {
        let mut line = Line::from("hello");
        line.drain(2..2); // no chars removed
        assert_eq!(line.content(), "hello");
        assert_eq!(line.len(), 5);
        assert_eq!(line.mem(), 5);
    }
}
