mod filter;

use crate::ceos::command::filter::Filter;
use crate::ceos::Ceos;
use crate::textarea::buffer::Buffer;
use log::warn;

impl Ceos {
    pub(crate) fn execute_command(&mut self, command: &str) {
        warn!("Execute command {}", command);
        if command.starts_with("filter ") {
            let params = command.strip_prefix("filter ").unwrap();
            let filter = Filter::from(params);
            filter.execute(self.textarea.buffer_mut())
        } else if command == "trim" {
            self.textarea.buffer_mut().trim_deleted_lines();
        }
    }
}

pub(crate) trait Command {
    fn execute(&self, buffer: &mut Buffer);
}
