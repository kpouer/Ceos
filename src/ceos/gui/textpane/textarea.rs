use std::sync::mpsc::Sender;

use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{FontId, Stroke, StrokeKind};
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, InputState, Response, Widget};
use log::info;

use crate::ceos::command::Command;
use crate::ceos::command::search::Search;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::event::Event;
use crate::event::Event::{ClearCommand, NewFont, OpenFile, SetCommand};

#[derive(Debug)]
pub(crate) struct TextArea<'a> {
    textarea_properties: &'a mut TextAreaProperties,
    current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
    drawing_rect: Rect,
    rect: Rect,
    theme: &'a Theme,
    sender: &'a Sender<Event>,
    search: &'a Search,
}

impl<'a> TextArea<'a> {
    pub(crate) const fn new(
        textarea_properties: &'a mut TextAreaProperties,
        current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
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
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let text_bounds = self.textarea_properties.text_bounds();
        let (rect, mut response) =
            ui.allocate_exact_size(text_bounds, egui::Sense::click_and_drag());

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Text);
        }

        if response.clicked() || response.drag_started() {
            self.sender.send(ClearCommand).unwrap();
            self.update_caret_position(rect, &response);
            response.mark_changed();
        } else if (response.dragged() || response.drag_stopped())
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            let column = self
                .textarea_properties
                .x_to_column(pointer_pos.x - rect.left());
            let caret_column = self.textarea_properties.caret_position.column;
            if caret_column > column {
                self.sender
                    .send(SetCommand(format!("{column}..{caret_column}")))
                    .unwrap();
            } else {
                self.sender
                    .send(SetCommand(format!("{caret_column}..{column}")))
                    .unwrap();
            }
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                self.drawing_rect,
                0.0,
                self.theme.background,
                Stroke::NONE,
                StrokeKind::Inside,
            );
            ui.set_height(self.textarea_properties.text_height());
            let mut drawing_pos = Pos2::new(ui.max_rect().left(), ui.clip_rect().top());
            self.handle_input(ui.ctx(), self.drawing_rect.left_top());
            let row_range = self.textarea_properties.get_row_range_for_rect(self.rect);
            // Ensure the buffer has decompressed the groups needed for the visible range
            self.textarea_properties
                .buffer
                .prepare_range_for_read(row_range.clone());
            row_range.into_iter().for_each(|line| {
                if self.search.has_results() {
                    self.search.paint_line(
                        ui,
                        self.theme,
                        self.textarea_properties,
                        line,
                        drawing_pos,
                    );
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
        }

        response
    }
}

impl TextArea<'_> {
    fn update_caret_position(&mut self, rect: Rect, response: &Response) {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let column = self
                .textarea_properties
                .x_to_column(pointer_pos.x - rect.left());
            let line = self
                .textarea_properties
                .y_to_line(pointer_pos.y - rect.top());
            self.textarea_properties.caret_position = Position { column, line };
        }
    }

    fn handle_input(&self, ctx: &Context, top_left: Pos2) {
        ctx.input(|i| {
            self.handle_dropped_file(i);
            let textarea_properties = &self.textarea_properties;
            if i.pointer.primary_clicked()
                && let Some(mut pos) = i.pointer.latest_pos()
            {
                pos.x -= top_left.x;
                pos.y -= top_left.y;
                let (column, line) = textarea_properties.point_to_text_position(pos);
                info!("point to column:{column} line:{line},  topleft {top_left}, pos {pos}");
                // textarea_properties.caret_position = Position { column, line };
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

    fn handle_dropped_file(&self, i: &InputState) {
        if let Some(file) = i.raw.dropped_files.first()
            && let Some(path) = &file.path
        {
            self.sender.send(OpenFile(path.to_owned())).unwrap();
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
