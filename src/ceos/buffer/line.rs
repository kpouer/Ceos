#[derive(Default, Debug)]
pub(crate) struct Line {
    pub(crate) content: String,
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self { content }
    }
}

impl From<&str> for Line {
    fn from(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

impl Line {
    pub(crate) fn len(&self) -> usize {
        self.content.len()
    }
}
