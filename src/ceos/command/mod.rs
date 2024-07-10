use std::fmt::Display;

use log::{debug, warn};

use crate::ceos::Ceos;
use crate::ceos::command::filter::Filter;
use crate::textarea::buffer::Buffer;
use crate::textarea::renderer::Renderer;

mod filter;

impl Ceos {
    pub(crate) fn try_command(&mut self) {
        debug!("Try command {}", self.command_buffer);
        if let Ok(command) = Filter::try_from(self.command_buffer.as_str()) {
            debug!("Found command {}", command);
            self.current_command = Some(Box::new(command));
        }
    }
}

impl Ceos {
    pub(crate) fn execute_command(&mut self) {
        if let Some(command) = self.current_command.take() {
            warn!("Execute command {}", command);
            command.execute(&mut self.textarea.buffer_mut());
        }
    }
}

pub(crate) trait Command: Renderer + Display {
    fn execute(&self, buffer: &mut Buffer);
}
