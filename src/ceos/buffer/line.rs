use std::ops::RangeBounds;

#[derive(Default, Debug)]
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
