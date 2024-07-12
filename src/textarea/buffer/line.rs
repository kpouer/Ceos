use crate::textarea::buffer::line_status::LineStatus;

#[derive(Default, Debug)]
pub(crate) struct Line {
    content: String,
    status: LineStatus,
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self {
            content,
            ..Default::default()
        }
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

    pub(crate) fn status(&self) -> &LineStatus {
        &self.status
    }

    pub(crate) fn set_status(&mut self, line_status: LineStatus) {
        self.status = line_status;
    }
}
