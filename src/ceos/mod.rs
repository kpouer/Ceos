use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::ceos::command::Command;
use crate::ceos::gui::frame_history::FrameHistory;
use crate::ceos::gui::searchpanel::SearchPanel;
use crate::event::Event;
use crate::event::Event::BufferLoaded;
use egui::Context;
use gui::textpane::textareaproperties::TextAreaProperties;
use gui::theme::Theme;
use log::warn;

pub(crate) mod buffer;
pub(crate) mod command;
pub(crate) mod gui;
mod syntax;
mod tools;

pub(crate) struct Ceos {
    textarea_properties: TextAreaProperties,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    command_buffer: String,
    current_command: Option<Box<dyn Command>>,
    frame_history: FrameHistory,
    search_panel: SearchPanel,
    theme: Theme,
    initialized: bool,
    loading_progress: Option<LoadingProgress>,
}

struct LoadingProgress {
    path: PathBuf,
    current: usize,
    size: usize,
}

impl Default for Ceos {
    fn default() -> Self {
        let (user_input_sender, user_input_receiver) = channel::<Event>();
        let search_panel = SearchPanel::new(user_input_sender.clone());
        Self {
            sender: user_input_sender,
            receiver: user_input_receiver,
            textarea_properties: Default::default(),
            command_buffer: String::new(),
            current_command: None,
            frame_history: Default::default(),
            search_panel,
            theme: Theme::default(),
            initialized: false,
            loading_progress: None,
        }
    }
}

impl Ceos {
    pub(crate) fn process_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::OpenFile(path) => self.open_file(path),
            Event::BufferLoadingStarted(path, size) => {
                self.loading_progress = Some(LoadingProgress {
                    path,
                    current: 0,
                    size,
                })
            }
            Event::BufferLoading(path, current, size) => match &mut self.loading_progress {
                None => warn!("Unexpected BufferLoading event"),
                Some(loading_progress) => {
                    loading_progress.current = current;
                }
            },
            BufferLoaded(buffer) => {
                self.loading_progress = None;
                self.textarea_properties.set_buffer(buffer);
            }
            Event::BufferClosed => self.textarea_properties.set_buffer(Default::default()),
            Event::GotoLine(goto) => goto.execute(ctx, &mut self.textarea_properties),
            Event::NewFont(font_id) => self.textarea_properties.set_font_id(font_id),
        }
    }
}
