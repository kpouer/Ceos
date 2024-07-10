#[derive(Default)]
pub(crate) struct BufferProperties {
    first_line: usize,
    horizontal_offset: usize,
}

impl BufferProperties {
    pub fn first_line(&self) -> usize {
        self.first_line
    }

    pub fn set_first_line(&mut self, first_line: usize) {
        self.first_line = first_line;
    }

    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    pub fn set_horizontal_offset(&mut self, horizontal_offset: usize) {
        self.horizontal_offset = horizontal_offset;
    }
}
