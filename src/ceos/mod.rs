use std::sync::mpsc::{channel, Receiver, Sender};

use crate::ceos::command::Command;
use crate::ceos::gui::frame_history::FrameHistory;
use crate::event::Event;
use crate::event::Event::BufferLoaded;
use anyhow::Error;
use buffer::Buffer;
use egui::Context;
use gui::textarea::textareaproperties::TextAreaProperties;
use gui::theme::Theme;

pub(crate) mod buffer;
pub(crate) mod command;
pub(crate) mod gui;
mod syntax;
mod tools;

pub(crate) struct Ceos {
    textarea: TextAreaProperties,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    command_buffer: String,
    current_command: Option<Box<dyn Command>>,
    frame_history: FrameHistory,
    theme: Theme,
    initialized: bool,
}

impl Ceos {
    pub(crate) fn process_event(&mut self, ctx: &Context, event: Event) {
        match event {
            BufferLoaded(buffer) => self.textarea.set_buffer(buffer),
            Event::BufferClosed => self.textarea.set_buffer(Default::default()),
            Event::GotoLine(goto) => goto.execute(ctx, &mut self.textarea),
            Event::NewFont(font_id) => self.textarea.set_font_id(font_id),
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
            frame_history: Default::default(),
            theme: Theme::default(),
            initialized: false,
        }
    }
}

impl TryFrom<&str> for Ceos {
    type Error = Error;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let buffer = Buffer::new_from_file(path.to_string())?;
        let textarea = buffer.into();
        Ok(Self {
            textarea,
            ..Default::default()
        })
    }
}
