use std::sync::mpsc::Sender;

use crate::ceos::command::Command;
use crate::ceos::command::search::Search;
use crate::ceos::gui::action::action_context::ActionContext;
use crate::ceos::gui::action::keyboard_handler::KeyboardHandler;
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
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{FontId, Stroke, StrokeKind};
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, EventFilter, InputState, KeyboardShortcut, Modifiers, Response, Ui, Widget};
use log::error;

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
    keyboard_handler: &'a KeyboardHandler,
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
        keyboard_handler: &'a KeyboardHandler,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            drawing_rect,
            virtual_rect,
            theme,
            sender,
            search,
            keyboard_handler,
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

        self.handle_mouse_interaction(rect, &mut response);

        if response.has_focus() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        vertical_arrows: true,
                        horizontal_arrows: true,
                        tab: true,
                        ..Default::default()
                    },
                )
            });
            self.handle_input(ui.ctx(), rect.min, true);
        }

        if ui.is_rect_visible(rect) {
            self.paint_content(ui, response.has_focus());
        }

        response
    }
}

const DRAG_STARTED_ID: &str = "drag_started";

impl TextArea<'_> {
    fn handle_mouse_interaction(&mut self, rect: Rect, response: &mut Response) {
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

    fn handle_click(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        let _ = self.sender.send(ClearCommand);
        response.request_focus();
        self.update_caret_position(rect, pointer_pos);
        response.mark_changed();
    }

    fn handle_double_click(&mut self, rect: Rect, response: &mut Response, pointer_pos: &Pos2) {
        self.handle_click(rect, response, pointer_pos);
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
                CaretRenderer.paint_line(
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

    fn handle_input(&mut self, ctx: &Context, _top_left: Pos2, has_focus: bool) {
        let mut scroll_offset = self.textarea_properties.scroll_offset;
        let line_height = self.textarea_properties.line_height;
        let rect_height = self.virtual_rect.height();

        let old_caret_position = self.textarea_properties.caret_position;

        ctx.input(|i| self.handle_dropped_file(i));
        if has_focus {
            let event_filter = EventFilter {
                vertical_arrows: true,
                horizontal_arrows: true,
                ..Default::default()
            };
            let events = ctx.input(|i| i.filtered_events(&event_filter));
            for event in events {
                match event {
                    egui::Event::Key {
                        pressed: true,
                        repeat: _,
                        key,
                        modifiers,
                        ..
                    } => {
                        let shortcut = KeyboardShortcut::new(modifiers, key);
                        if let Some(action) = self.keyboard_handler.get_action(&shortcut) {
                            let mut action_context = ActionContext::new(self.textarea_properties);
                            action.execute(&mut action_context);
                        }
                        ctx.input_mut(|i| i.consume_shortcut(&shortcut));
                    }
                    MouseWheel {
                        unit: _,
                        delta,
                        modifiers,
                    } => self.handle_mouse_wheel(&modifiers, &delta),
                    Zoom(delta) => self.handle_zoom(delta),
                    egui::Event::Copy => self.textarea_properties.copy(ctx),
                    egui::Event::Cut => self.textarea_properties.cut(ctx),
                    egui::Event::Paste(text) => self.textarea_properties.replace_selection(&text),
                    egui::Event::Text(text) => {
                        println!("text {}", text);
                        self.textarea_properties.replace_selection(&text)
                    }
                    _ => {}
                }
                ctx.input_mut(|i| i.events.clear());
            }
        }

        if old_caret_position != self.textarea_properties.caret_position {
            let caret_y = self.textarea_properties.caret_position.line as f32 * line_height;
            if caret_y < scroll_offset.y {
                scroll_offset.y = caret_y;
            } else if caret_y + line_height > scroll_offset.y + rect_height {
                scroll_offset.y = caret_y + line_height - rect_height;
            }

            let caret_x = self.textarea_properties.caret_position.column as f32
                * self.textarea_properties.char_width;
            let rect_width = self.virtual_rect.width();
            if caret_x < scroll_offset.x {
                scroll_offset.x = caret_x;
            } else if caret_x + self.textarea_properties.char_width > scroll_offset.x + rect_width {
                scroll_offset.x = caret_x + self.textarea_properties.char_width - rect_width;
            }

            self.textarea_properties.scroll_offset = scroll_offset;
        }
    }

    fn handle_dropped_file(&self, i: &InputState) {
        if let Some(file) = i.raw.dropped_files.first()
            && let Some(path) = &file.path
        {
            let _ = self.sender.send(OpenFile(path.to_owned()));
        }
    }

    fn handle_mouse_wheel(&self, modifiers: &Modifiers, delta: &Vec2) {
        #[cfg(target_os = "macos")]
        if modifiers.command {
            self.zoom(delta.y);
        }
        #[cfg(not(target_os = "macos"))]
        if modifiers.ctrl {
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

        let _ = self.sender.send(NewFont(FontId::new(
            new_font_size,
            egui::FontFamily::Monospace,
        )));
    }

    #[inline]
    fn visible_line_count(&self) -> usize {
        (self.virtual_rect.height() / self.textarea_properties.line_height) as usize
    }
}
