use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::command::Command;
use crate::ceos::command::filter::columnfilter::ColumnFilter;
use crate::ceos::command::filter::linedrop::LineDrop;
use crate::ceos::command::filter::linefilter::LineFilter;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::event::Event;
use log::{debug, info};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(crate) struct CommandState {
    sender: Sender<Event>,
    command_buffer: String,
    current_command: Option<Box<dyn Command + Send + Sync + 'static>>,
}

impl CommandState {
    pub(crate) fn new(sender: Sender<Event>) -> CommandState {
        Self {
            sender,
            command_buffer: String::new(),
            current_command: None,
        }
    }
}

impl CommandState {
    pub(crate) fn clear_command(&mut self) {
        self.command_buffer = String::new();
        self.current_command = None;
    }

    pub(crate) fn set_command_buffer(&mut self, command: String) {
        self.command_buffer = command;
        self.try_filter_command();
    }

    pub(crate) fn command_buffer(&mut self) -> &str {
        &self.command_buffer
    }

    pub(crate) fn command_buffer_mut(&mut self) -> &mut String {
        &mut self.command_buffer
    }

    pub(crate) fn current_command_mut(
        &mut self,
    ) -> &mut Option<Box<dyn Command + Send + Sync + 'static>> {
        &mut self.current_command
    }

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

    pub(crate) fn execute_command(&mut self, textarea_properties: &mut TextAreaProperties) {
        if let Some(command) = self.current_command.take() {
            info!("Execute command {}", command);
            let sender = self.sender.clone();
            let mut tmp_buffer = Buffer::new_empty_buffer(self.sender.clone());
            std::mem::swap(&mut tmp_buffer, &mut textarea_properties.buffer);
            // now the real buffer is sent to the command to be executed
            std::thread::spawn(move || {
                command.execute(&mut tmp_buffer);
                let _ = sender.send(Event::BufferLoaded(tmp_buffer));
            });
        } else if let Ok(command) = Event::try_from(self.command_buffer.as_str()) {
            let _ = self.sender.send(command);
        }
        self.clear_command();
    }
}
