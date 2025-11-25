#[derive(Default, Debug)]
pub(crate) struct Line {
    pub(crate) content: String,
}

impl<T: Into<String>> From<T> for Line {
    fn from(content: T) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Line {
    pub(crate) fn len(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn mem(&self) -> usize {
        self.content.capacity()
    }
}
