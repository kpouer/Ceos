use std::ops::RangeBounds;
use std::string::Drain;

#[derive(Default, Debug)]
pub(crate) struct Line {
    content: String,
}

impl<T: Into<String>> From<T> for Line {
    fn from(content: T) -> Self {
        Self {
            content: content.into(),
        }
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

    pub(crate) fn drain<R>(&mut self, range: R) -> Drain<'_>
    where
        R: RangeBounds<usize>,
    {
        self.content.drain(range)
    }
}
