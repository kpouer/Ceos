use crate::ceos::command::Command;
use crate::textarea::buffer::line::Line;
use crate::textarea::buffer::Buffer;

pub(crate) struct Filter<'a> {
    command: &'a str,
}

impl<'a> Filter<'a> {
    pub(crate) fn accept(&self, line: &Line) -> bool {
        line.content().contains(self.command)
    }
}

impl<'a> From<&'a str> for Filter<'a> {
    fn from(command: &'a str) -> Self {
        Self { command }
    }
}

impl Command for Filter<'_> {
    fn execute(&self, buffer: &mut Buffer) {
        for line in buffer.content_mut().iter_mut() {
            if !self.accept(line) {
                line.set_deleted();
            }
        }
        buffer.compute_total_length();
    }
}
