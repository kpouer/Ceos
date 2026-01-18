use std::sync::mpsc::Sender;

use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{FontId, Stroke, StrokeKind};
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, InputState, Response, Ui, Widget};
use log::info;

use crate::ceos::command::Command;
use crate::ceos::command::search::Search;
use crate::ceos::gui::textpane::interaction_mode::InteractionMode;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::selection::Selection;
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

        self.handle_interaction(rect, &mut response);

        if ui.is_rect_visible(rect) {
            self.paint_content(ui);
        }

        response
    }
}

const DRAG_STARTED_ID: &str = "drag_started";

impl TextArea<'_> {
    fn handle_interaction(&mut self, rect: Rect, response: &mut Response) {
        let Some(pointer_pos) = response.interact_pointer_pos() else {
            return;
        };
        if response.clicked() || response.drag_started() {
            let _ = self.sender.send(ClearCommand);
            self.update_caret_position(rect, &pointer_pos);
            response.mark_changed();
            if response.drag_started() {
                response.ctx.memory_mut(|m| {
                    m.data.insert_temp(
                        DRAG_STARTED_ID.into(),
                        self.textarea_properties.caret_position,
                    )
                });
            }
        } else if response.dragged() || response.drag_stopped() {
            response.mark_changed();
            match self.textarea_properties.interaction_mode {
                InteractionMode::Column => {
                    self.handle_drag_update_column(rect, response, pointer_pos)
                }
                InteractionMode::Selection => {
                    self.handle_drag_update_selection(rect, response, &pointer_pos)
                }
            }
            if response.drag_stopped() {
                response
                    .ctx
                    .memory_mut(|m| m.data.remove_temp::<Position>(DRAG_STARTED_ID.into()));
            }
            response.mark_changed();
        }
    }

    fn handle_drag_update_selection(
        &mut self,
        rect: Rect,
        response: &mut Response,
        pointer_pos: &Pos2,
    ) {
        let pointer_pos = self.build_position(rect, &pointer_pos);
        let drag_start_position = response.ctx.memory(|m| {
            m.data
                .get_temp(DRAG_STARTED_ID.into())
                .expect("there should be a drag_started")
        });
        let (start, end) = if drag_start_position < pointer_pos {
            (drag_start_position, pointer_pos)
        } else {
            (pointer_pos, drag_start_position)
        };
        self.textarea_properties.selection = Some(Selection { start, end });
    }

    fn handle_drag_update_column(
        &mut self,
        rect: Rect,
        response: &mut Response,
        pointer_pos: Pos2,
    ) {
        let column = self
            .textarea_properties
            .x_to_column(pointer_pos.x - rect.left());
        let drag_start_position = response.ctx.memory(|m| {
            m.data
                .get_temp(DRAG_STARTED_ID.into())
                .expect("there should be a drag_started")
        });
        let start = column.min(drag_start_position);
        let end = column.max(drag_start_position);
        let _ = self.sender.send(SetCommand(format!("{start}..{end}")));
    }

    fn paint_content(&mut self, ui: &mut Ui) {
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
    }
}

impl TextArea<'_> {
    fn update_caret_position(&mut self, rect: Rect, pos: &Pos2) {
        self.textarea_properties.caret_position = self.build_position(rect, pos);
    }

    /// ```rust
    /// Builds a `Position` object based on the interaction pointer's position within a given rectangular area.
    ///
    /// # Arguments
    ///
    /// * `rect` - A `Rect` representing the area in which the position is being calculated.
    /// * `pos` - A `Pos2` object containing coordinates on screen.
    ///
    /// # Returns
    ///
    /// Returns a `Position`:
    /// - `Position` the position in the text.
    ///
    /// The `Position` object represents the calculated column and line, using `textarea_properties`
    /// to map the pointer's x and y coordinates relative to the `rect`.
    fn build_position(&self, rect: Rect, pos: &Pos2) -> Position {
        let column = self.textarea_properties.x_to_column(pos.x - rect.left());
        let line = self.textarea_properties.y_to_line(pos.y - rect.top());
        Position { column, line }
    }

    fn handle_input(&self, ctx: &Context, top_left: Pos2) {
        ctx.input(|i| {
            self.handle_dropped_file(i);
            if i.pointer.primary_clicked()
                && let Some(mut pos) = i.pointer.latest_pos()
            {
                pos.x -= top_left.x;
                pos.y -= top_left.y;
                let position = self.textarea_properties.point_to_text_position(pos);
                info!("point to {position}, topleft {top_left}, pos {pos}");
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
