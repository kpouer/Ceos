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

impl Line {
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    pub(crate) fn status(&self) -> &LineStatus {
        &self.status
    }

    pub(crate) fn set_status(&mut self, line_status: LineStatus) {
        self.status = line_status;
    }
}
