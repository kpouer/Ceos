use std::sync::mpsc::Sender;
use std::thread;

use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{FontId, Stroke};
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, InputState, Widget};
use log::{info, warn};

use crate::ceos::buffer::Buffer;
use crate::ceos::command::search::Search;
use crate::ceos::command::Command;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::event::Event;
use crate::event::Event::{BufferClosed, BufferLoaded, NewFont};

pub(crate) struct TextArea<'a> {
    textarea_properties: &'a TextAreaProperties,
    current_command: &'a Option<Box<dyn Command>>,
    drawing_rect: Rect,
    rect: Rect,
    theme: &'a Theme,
    sender: &'a Sender<Event>,
    search: &'a Search,
}

impl<'a> TextArea<'a> {
    pub(crate) fn new(
        textarea_properties: &'a TextAreaProperties,
        current_command: &'a Option<Box<dyn Command>>,
        drawing_rect: Rect,
        rect: Rect,
        theme: &'a Theme,
        sender: &'a Sender<Event>,
        search: &'a Search,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            drawing_rect,
            rect,
            theme,
            sender,
            search,
        }
    }
}

impl Widget for &mut TextArea<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.painter()
            .rect(self.drawing_rect, 0.0, self.theme.background, Stroke::NONE);
        ui.set_height(self.textarea_properties.text_height());
        let mut drawing_pos = Pos2::new(ui.max_rect().left(), ui.clip_rect().top());
        self.handle_input(ui.ctx(), self.drawing_rect.left_top());
        let row_range = self.textarea_properties.get_row_range_for_rect(self.rect);
        row_range.into_iter().for_each(|line| {
            if !self.search.lines.is_empty() {
                self.search
                    .paint_line(ui, self.theme, self.textarea_properties, line, drawing_pos);
            }
            if let Some(filter_renderer) = &self.current_command {
                filter_renderer.paint_line(
                    ui,
                    self.theme,
                    self.textarea_properties,
                    line,
                    drawing_pos,
                );
            }

            self.textarea_properties.renderer_manager.paint_line(
                ui,
                self.theme,
                self.textarea_properties,
                line,
                drawing_pos,
            );

            drawing_pos.y += self.textarea_properties.line_height;
        });

        let text_bounds = self.textarea_properties.text_bounds();
        let (_, response) = ui.allocate_exact_size(text_bounds, egui::Sense::click_and_drag());
        response
    }
}

impl TextArea<'_> {
    fn handle_input(&mut self, ctx: &Context, top_left: Pos2) {
        ctx.input(|i| {
            self.handle_dropped_file(i);
            let textarea_properties = &self.textarea_properties;
            if i.pointer.primary_clicked() {
                if let Some(mut pos) = i.pointer.latest_pos() {
                    pos.x -= top_left.x;
                    pos.y -= top_left.y;
                    let (column, line) = textarea_properties.point_to_text_position(pos);
                    info!("point to column:{column} line:{line},  topleft {top_left}, pos {pos}");
                    // textarea_properties.caret_position = Position { column, line };
                }
            }

            i.events.iter().for_each(|event| match event {
                MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => self.handle_mouse_wheel(i, delta),
                Zoom(delta) => self.handle_zoom(*delta),
                _ => {}
            })
        });
    }

    fn handle_dropped_file(&mut self, i: &InputState) {
        if let Some(file) = i.raw.dropped_files.first() {
            if let Some(path) = &file.path {
                let path = path.to_string_lossy();
                let path = path.to_string();
                let sender = self.sender.clone();
                thread::spawn(move || {
                    sender.send(BufferClosed).unwrap();
                    match Buffer::new_from_file(path) {
                        Ok(buffer) => sender.send(BufferLoaded(buffer)).unwrap(),
                        Err(e) => warn!("{:?}", e),
                    }
                });
            }
        }
    }

    fn handle_mouse_wheel(&self, input_state: &InputState, delta: &Vec2) {
        #[cfg(target_os = "macos")]
        if input_state.modifiers.command {
            self.zoom(delta.y);
        }
        #[cfg(not(target_os = "macos"))]
        if input_state.modifiers.ctrl {
            self.zoom(delta.y);
        }
    }

    fn handle_zoom(&self, delta: f32) {
        if delta < 1.0 {
            self.zoom(-delta);
        } else if delta > 1.0 {
            self.zoom(delta);
        }
    }

    fn zoom(&self, delta: f32) {
        let current_font_size = self.textarea_properties.font_id.size;
        let new_font_size = current_font_size + delta;
        if new_font_size < 1.0 {
            return;
        }

        self.sender
            .send(NewFont(FontId::new(
                new_font_size,
                egui::FontFamily::Monospace,
            )))
            .unwrap()
    }
}
