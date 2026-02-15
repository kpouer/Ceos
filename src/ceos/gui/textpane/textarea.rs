use std::sync::mpsc::Sender;

use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{FontId, Stroke, StrokeKind};
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, InputState, Response, Ui, Widget};
use log::info;

use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::command::Command;
use crate::ceos::command::search::Search;
use crate::ceos::gui::textpane::interaction_mode::InteractionMode;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::renderer::caret_renderer::CaretRenderer;
use crate::ceos::gui::textpane::selection::Selection;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::tools::text_tool::TextTool;
use crate::event::Event;
use crate::event::Event::{ClearCommand, NewFont, OpenFile, SetCommand};

#[derive(Debug)]
pub(crate) struct TextArea<'a> {
    textarea_properties: &'a mut TextAreaProperties,
    current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
    // The rectangle on screen occupied by the text area
    drawing_rect: Rect,
    // the rectangle that is drawn in the viewport
    virtual_rect: Rect,
    theme: &'a Theme,
    sender: &'a Sender<Event>,
    search: &'a Search,
}

impl<'a> TextArea<'a> {
    pub(crate) const fn new(
        textarea_properties: &'a mut TextAreaProperties,
        current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
        drawing_rect: Rect,
        virtual_rect: Rect,
        theme: &'a Theme,
        sender: &'a Sender<Event>,
        search: &'a Search,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            drawing_rect,
            virtual_rect,
            theme,
            sender,
            search,
        }
    }
}

impl Widget for &mut TextArea<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let mut text_bounds = self.textarea_properties.text_bounds();
        text_bounds.x = self.virtual_rect.width().max(text_bounds.x);
        text_bounds.y = self.virtual_rect.height().max(text_bounds.y);
        let (rect, mut response) =
            ui.allocate_exact_size(text_bounds, egui::Sense::click_and_drag());

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Text);
        }

        self.handle_interaction(rect, &mut response);

        if ui.is_rect_visible(rect) {
            self.paint_content(ui, response.has_focus());
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
        if response.double_clicked() {
            self.handle_double_click(rect, response, &pointer_pos);
        } else if response.clicked() {
            self.handle_click(rect, response, &pointer_pos);
        } else if response.drag_started() {
            self.handle_drag_start(rect, response, &pointer_pos);
        } else if response.dragged() {
            self.handle_dragged(rect, response, &pointer_pos);
        } else if response.drag_stopped() {
            self.handle_drag_stopped(rect, response, &pointer_pos);
        }
    }

    fn handle_double_click(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        let _ = self.sender.send(ClearCommand);
        self.update_caret_position(rect, pointer_pos);
        let caret_position = self.textarea_properties.caret_position;
        let text = self
            .textarea_properties
            .buffer
            .line_text(caret_position.line);
        let text_tool = TextTool::new(text);
        let start_col = text_tool.find_word_start(caret_position.column);

        let end_col = text_tool.find_word_end(caret_position.column);
        self.textarea_properties.selection = Some(Selection::new(
            Position {
                line: caret_position.line,
                column: start_col,
            },
            Position {
                line: caret_position.line,
                column: end_col,
            },
        ));
        response.mark_changed();
    }

    fn handle_click(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        let _ = self.sender.send(ClearCommand);
        response.request_focus();
        self.update_caret_position(rect, pointer_pos);
        response.mark_changed();
    }

    fn handle_drag_start(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        self.handle_click(rect, response, pointer_pos);
        response.ctx.memory_mut(|m| {
            m.data.insert_temp(
                DRAG_STARTED_ID.into(),
                self.textarea_properties.caret_position,
            )
        });
    }

    fn handle_dragged(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        response.mark_changed();
        let drag_start_position = response.ctx.memory(|m| {
            m.data
                .get_temp::<Position>(DRAG_STARTED_ID.into())
                .expect("there should be a drag_started")
        });
        match self.textarea_properties.interaction_mode {
            InteractionMode::Column => {
                self.handle_drag_update_column(rect, drag_start_position, pointer_pos)
            }
            InteractionMode::Selection => {
                self.handle_drag_update_selection(rect, drag_start_position, pointer_pos)
            }
        }
    }

    fn handle_drag_stopped(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        self.handle_dragged(rect, response, pointer_pos);
        response
            .ctx
            .memory_mut(|m| m.data.remove_temp::<Position>(DRAG_STARTED_ID.into()));
    }

    fn handle_drag_update_selection(
        &mut self,
        rect: Rect,
        drag_start_position: Position,
        pointer_pos: &Pos2,
    ) {
        let pointer_pos = self.build_position(rect, pointer_pos);
        let (start, end) = if drag_start_position < pointer_pos {
            (drag_start_position, pointer_pos)
        } else {
            (pointer_pos, drag_start_position)
        };
        self.textarea_properties.selection = Some(Selection::new(start, end));
    }

    fn handle_drag_update_column(
        &mut self,
        rect: Rect,
        drag_start_position: Position,
        pointer_pos: &Pos2,
    ) {
        let column = self
            .textarea_properties
            .x_to_column(pointer_pos.x - rect.left());
        let start = column.min(drag_start_position.column);
        let end = column.max(drag_start_position.column);
        let _ = self.sender.send(SetCommand(format!("{start}..{end}")));
    }

    fn paint_content(&mut self, ui: &mut Ui, has_focus: bool) {
        ui.painter().rect(
            self.drawing_rect,
            0.0,
            self.theme.background,
            Stroke::NONE,
            StrokeKind::Inside,
        );
        ui.set_height(self.textarea_properties.text_height());
        let mut drawing_pos = Pos2::new(ui.max_rect().left(), ui.clip_rect().top());
        self.handle_input(ui.ctx(), self.drawing_rect.left_top(), has_focus);
        let row_range = self
            .textarea_properties
            .get_row_range_for_rect(self.virtual_rect);
        if row_range.is_empty() {
            if has_focus {
                CaretRenderer::default().paint_line(
                    ui,
                    self.theme,
                    self.textarea_properties,
                    0,
                    drawing_pos,
                    true,
                );
            }
            return;
        }
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
                    has_focus,
                );
            }
            if let Some(filter_renderer) = &self.current_command {
                filter_renderer.paint_line(
                    ui,
                    self.theme,
                    self.textarea_properties,
                    line,
                    drawing_pos,
                    has_focus,
                );
            }

            self.textarea_properties.renderer_manager.paint_line(
                ui,
                self.theme,
                self.textarea_properties,
                line,
                drawing_pos,
                has_focus,
            );

            drawing_pos.y += self.textarea_properties.line_height;
        });
    }
}

impl TextArea<'_> {
    fn update_caret_position(&mut self, rect: Rect, pos: &Pos2) {
        // ensure the new caret position is within the bounds of the text area

        let mut new_caret_position = self.build_position(rect, pos);
        new_caret_position.line = new_caret_position
            .line
            .min(self.textarea_properties.buffer.line_count() - 1);
        new_caret_position.column = new_caret_position.column.min(
            self.textarea_properties
                .buffer
                .line_text(new_caret_position.line)
                .len(),
        );
        self.textarea_properties.caret_position = new_caret_position;
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

    fn handle_input(&mut self, ctx: &Context, top_left: Pos2, has_focus: bool) {
        let mut caret_position = self.textarea_properties.caret_position;
        let mut scroll_offset = self.textarea_properties.scroll_offset;
        let line_height = self.textarea_properties.line_height;
        let rect_height = self.virtual_rect.height();
        let line_count = self.textarea_properties.buffer.line_count();

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

            if has_focus {
                i.events.iter().for_each(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: _,
                        modifiers: _,
                        ..
                    } => self.handle_key_event(&mut caret_position, line_count, i, key),
                    egui::Event::Text(text) => {
                        self.delete_selection();
                        for ch in text.chars() {
                            if ch == '\r' || ch == '\x08' || ch == '\x7f' {
                                continue;
                            }
                            self.textarea_properties.buffer.insert_char(
                                caret_position.line,
                                caret_position.column,
                                ch,
                            );
                            if ch == '\n' {
                                caret_position.line += 1;
                                caret_position.column = 0;
                            } else {
                                caret_position.column += 1;
                            }
                        }
                    }
                    MouseWheel {
                        unit: _,
                        delta,
                        modifiers: _,
                    } => self.handle_mouse_wheel(i, delta),
                    Zoom(delta) => self.handle_zoom(*delta),
                    _ => {}
                });
            }
        });

        if self.textarea_properties.caret_position != caret_position {
            self.textarea_properties.caret_position = caret_position;

            // S'assurer que le curseur est visible après le déplacement
            let caret_y = caret_position.line as f32 * line_height;
            if caret_y < scroll_offset.y {
                scroll_offset.y = caret_y;
            } else if caret_y + line_height > scroll_offset.y + rect_height {
                scroll_offset.y = caret_y + line_height - rect_height;
            }

            let caret_x = caret_position.column as f32 * self.textarea_properties.char_width;
            let rect_width = self.virtual_rect.width();
            if caret_x < scroll_offset.x {
                scroll_offset.x = caret_x;
            } else if caret_x + self.textarea_properties.char_width > scroll_offset.x + rect_width {
                scroll_offset.x = caret_x + self.textarea_properties.char_width - rect_width;
            }

            self.textarea_properties.scroll_offset = scroll_offset;
        }
    }

    /// Delete the selection content if there is one.
    fn delete_selection(&mut self) {
        if let Some(selection) = self.textarea_properties.selection.take() {
            self.textarea_properties.buffer.delete_range(TextRange::from(&selection));
            self.textarea_properties.caret_position = selection.start;
        }
    }

    fn handle_key_event(
        &mut self,
        caret_position: &mut Position,
        line_count: usize,
        i: &InputState,
        key: &egui::Key,
    ) {
        match key {
            egui::Key::Home => {
                if Self::is_control_pressed(i) {
                    caret_position.line = 0;
                    caret_position.column = 0;
                } else {
                    caret_position.column = 0;
                }
            }
            egui::Key::End => {
                if Self::is_control_pressed(i) {
                    caret_position.line = line_count.saturating_sub(1);
                }
                let line_text = self
                    .textarea_properties
                    .buffer
                    .line_text(caret_position.line);
                caret_position.column = line_text.len();
            }
            egui::Key::ArrowLeft => {
                caret_position.column = caret_position.column.saturating_sub(1);
            }
            egui::Key::ArrowRight => {
                caret_position.column = self
                    .textarea_properties
                    .buffer
                    .line_text(caret_position.line)
                    .len()
                    .min(caret_position.column.saturating_add(1));
            }
            egui::Key::ArrowUp => {
                caret_position.line = caret_position.line.saturating_sub(1);
                self.textarea_properties.set_first_line(caret_position.line);
            }
            egui::Key::ArrowDown => {
                caret_position.line = self
                    .textarea_properties
                    .buffer
                    .line_count()
                    .min(caret_position.line + 1);
                self.textarea_properties.set_first_line(caret_position.line);
            }
            egui::Key::PageUp => {
                let visible_lines = self.visible_line_count();
                if caret_position.line > visible_lines {
                    caret_position.line -= visible_lines;
                } else {
                    caret_position.line = 0;
                }
                self.textarea_properties.set_first_line(caret_position.line);
            }
            egui::Key::PageDown => {
                let visible_lines = self.visible_line_count();
                caret_position.line =
                    (caret_position.line + visible_lines).min(line_count.saturating_sub(1));
                self.textarea_properties.set_first_line(caret_position.line);
            }
            egui::Key::Delete | egui::Key::Backspace => {
                if let Some(selection) = self.textarea_properties.selection.take() {
                    let mut start = selection.start;
                    let mut end = selection.end;
                    if start > end {
                        std::mem::swap(&mut start, &mut end);
                    }
                    let range = TextRange::new(start.line, start.column, end.line, end.column);
                    self.textarea_properties.buffer.delete_range(range);
                    *caret_position = start;
                } else if *key == egui::Key::Backspace {
                    if caret_position.column > 0 {
                        let range = TextRange::new(
                            caret_position.line,
                            caret_position.column - 1,
                            caret_position.line,
                            caret_position.column,
                        );
                        self.textarea_properties.buffer.delete_range(range);
                        caret_position.column -= 1;
                    } else if caret_position.line > 0 {
                        let prev_line_idx = caret_position.line - 1;
                        let prev_line_len = self
                            .textarea_properties
                            .buffer
                            .line_text(prev_line_idx)
                            .len();
                        let range =
                            TextRange::new(prev_line_idx, prev_line_len, caret_position.line, 0);
                        self.textarea_properties.buffer.delete_range(range);
                        caret_position.line = prev_line_idx;
                        caret_position.column = prev_line_len;
                    }
                } else if *key == egui::Key::Delete {
                    let line_len = self
                        .textarea_properties
                        .buffer
                        .line_text(caret_position.line)
                        .len();
                    if caret_position.column < line_len {
                        let range = TextRange::new(
                            caret_position.line,
                            caret_position.column,
                            caret_position.line,
                            caret_position.column + 1,
                        );
                        self.textarea_properties.buffer.delete_range(range);
                    } else if caret_position.line + 1 < line_count {
                        let range = TextRange::new(
                            caret_position.line,
                            line_len,
                            caret_position.line + 1,
                            0,
                        );
                        self.textarea_properties.buffer.delete_range(range);
                    }
                }
            }
            egui::Key::Enter => {
                self.textarea_properties
                    .buffer
                    .insert_newline(caret_position.line, caret_position.column);
                caret_position.line += 1;
                caret_position.column = 0;
            }
            _ => {}
        }
    }

    fn is_control_pressed(i: &InputState) -> bool {
        if cfg!(target_os = "macos") {
            i.modifiers.command
        } else {
            i.modifiers.ctrl
        }
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

    fn visible_line_count(&self) -> usize {
        (self.virtual_rect.height() / self.textarea_properties.line_height) as usize
    }
}
