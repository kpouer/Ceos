use std::fmt::Display;

use log::{debug, info};

use buffer::columnfilter::ColumnFilter;
use buffer::linefilter::LineFilter;

use crate::ceos::command::buffer::linedrop::LineDrop;
use crate::ceos::textarea::buffer::Buffer;
use crate::ceos::textarea::renderer::Renderer;
use crate::ceos::Ceos;
use crate::event::Event;

mod buffer;
pub(crate) mod direct;

impl Ceos {
    pub(crate) fn try_filter_command(&mut self) {
        let command_str = self.command_buffer.as_str();
        if let Ok(command) = LineFilter::try_from(command_str) {
            self.current_command = Some(Box::new(command));
        } else if let Ok(command) = ColumnFilter::try_from(command_str) {
            self.current_command = Some(Box::new(command));
        } else if let Ok(command) = LineDrop::try_from(command_str) {
            self.current_command = Some(Box::new(command));
        } else {
            self.current_command = None;
        }

        if let Some(command) = &self.current_command {
            debug!("Found command {}", command);
        }
    }

    pub(crate) fn execute_command(&mut self) {
        if let Some(command) = self.current_command.take() {
            info!("Execute command {}", command);
            command.execute(self.textarea.buffer_mut());
        } else if let Ok(command) = Event::try_from(self.command_buffer.as_str()) {
            self.sender.send(command).unwrap();
        }
    }
}

pub(crate) trait Command: Renderer + Display {
    fn execute(&self, buffer: &mut Buffer);
}
