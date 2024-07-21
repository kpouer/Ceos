use std::fmt::Display;

use log::{debug, info};

use buffer::columnfilter::ColumnFilter;
use buffer::linefilter::LineFilter;

use crate::ceos::command::direct::DirectTextAreaCommand;
use crate::ceos::textarea::buffer::Buffer;
use crate::ceos::textarea::renderer::Renderer;
use crate::ceos::Ceos;

mod buffer;
mod direct;

impl Ceos {
    pub(crate) fn try_filter_command(&mut self) {
        let command_str = self.command_buffer.as_str();
        if let Ok(command) = LineFilter::try_from(command_str) {
            self.current_command = Some(Box::new(command));
        } else if let Ok(command) = ColumnFilter::try_from(command_str) {
            self.current_command = Some(Box::new(command));
        } else {
            self.current_command = None;
        }

        if let Some(command) = &self.current_command {
            debug!("Found command {}", command);
        }
    }

    fn try_direct_command(&mut self) {
        if let Ok(command) = DirectTextAreaCommand::try_from(self.command_buffer.as_str()) {
            command.execute(self.command_buffer.as_str(), &mut self.textarea);
        }
    }

    pub(crate) fn execute_command(&mut self) {
        if let Some(command) = self.current_command.take() {
            info!("Execute command {}", command);
            command.execute(self.textarea.buffer_mut());
        } else {
            self.try_direct_command();
        }
    }
}

pub(crate) trait Command: Renderer + Display {
    fn execute(&self, buffer: &mut Buffer);
}
