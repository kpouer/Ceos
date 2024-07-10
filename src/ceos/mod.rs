use std::sync::mpsc::{channel, Receiver, Sender};

use anyhow::Error;

use crate::ceos::command::Command;
use crate::event::Event;
use crate::event::Event::BufferLoaded;
use crate::textarea::buffer::Buffer;
use crate::textarea::textarea::TextArea;

pub(crate) mod command;
mod gui;

pub(crate) struct Ceos {
    textarea: TextArea,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    command_buffer: String,
    current_command: Option<Box<dyn Command>>
}

impl Ceos {
    pub(crate) fn process_event(&mut self, event: Event) {
        match event {
            BufferLoaded(buffer) => self.textarea.set_buffer(buffer),
        }
    }
}

impl Default for Ceos {
    fn default() -> Self {
        let (user_input_sender, user_input_receiver) = channel::<Event>();
        Self {
            sender: user_input_sender,
            receiver: user_input_receiver,
            textarea: Default::default(),
            command_buffer: String::new(),
            current_command: None,
        }
    }
}

impl TryFrom<&str> for Ceos {
    type Error = Error;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let buffer = Buffer::try_from(path.to_string())?;
        let textarea = buffer.into();
        Ok(Self {
            textarea,
            ..Default::default()
        })
    }
}
