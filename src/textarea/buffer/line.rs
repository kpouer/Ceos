#[derive(Default, Debug)]
pub(crate) struct Line {
    content: String,
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self { content }
    }
}

impl From<&str> for Line {
    fn from(content: &str) -> Self {
        Self::from(content.to_string())
    }
}

impl Line {
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    pub(crate) fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}
