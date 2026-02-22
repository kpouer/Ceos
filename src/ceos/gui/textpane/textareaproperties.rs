use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::buffer::text_range::TextRange;
use crate::ceos::gui::textpane::gutter;
use crate::ceos::gui::textpane::interaction_mode::InteractionMode;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::renderer::caret_renderer::CaretRenderer;
use crate::ceos::gui::textpane::renderer::renderer_manager::{
    CARET_LAYER, RendererManager, SELECTION_LAYER, TEXT_LAYER,
};
use crate::ceos::gui::textpane::renderer::selection_renderer::SelectionRenderer;
use crate::ceos::gui::textpane::renderer::text_renderer::TextRenderer;
use crate::ceos::gui::textpane::selection::Selection;
use crate::event::Event;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::FontId;
use log::{debug, info};
use std::cmp;
use std::ops::Range;
use std::sync::mpsc::Sender;

pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 16.0;
pub(crate) const MAX_COPY_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct TextAreaProperties {
    pub(crate) buffer: Buffer,
    pub(crate) line_height: f32,
    pub(crate) font_id: FontId,
    pub(crate) char_width: f32,
    pub(crate) renderer_manager: RendererManager,
    pub(crate) caret_position: Position,
    pub(crate) selection: Option<Selection>,
    pub(crate) interaction_mode: InteractionMode,
    pub(crate) scroll_offset: Vec2,
}

impl TextAreaProperties {
    pub(crate) fn new(sender: Sender<Event>) -> TextAreaProperties {
        let font_id = FontId::new(DEFAULT_LINE_HEIGHT, egui::FontFamily::Monospace);
        let mut renderer_manager = RendererManager::default();
        renderer_manager.add_renderer(TEXT_LAYER, Box::new(TextRenderer::new(font_id.clone())));
        renderer_manager.add_renderer(SELECTION_LAYER, Box::new(SelectionRenderer {}));
        renderer_manager.add_renderer(CARET_LAYER, Box::new(CaretRenderer));
        Self {
            buffer: Buffer::new_empty_buffer(sender),
            renderer_manager,
            line_height: DEFAULT_LINE_HEIGHT,
            font_id,
            char_width: 0.0,
            caret_position: Position::default(),
            selection: None,
            interaction_mode: InteractionMode::Selection,
            scroll_offset: Vec2::ZERO,
        }
    }

    pub(crate) fn set_interaction_mode(&mut self, mode: InteractionMode) {
        self.interaction_mode = mode;
        if mode == InteractionMode::Column {
            self.selection = None;
        }
    }

    pub(crate) fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id.clone();
        self.char_width = 0.0;
        self.line_height = font_id.size;
        self.renderer_manager.set_font_id(font_id);
    }

    pub(crate) fn set_buffer(&mut self, buffer: Buffer) {
        info!(
            "set buffer: {:?}, line count: {}",
            buffer.path,
            buffer.line_count()
        );
        self.buffer = buffer
    }

    /// ```
    /// Sets the first visible line of the buffer in the view.
    ///
    /// This function adjusts the scroll offset based on the specified line,
    /// ensuring that the given line becomes the first visible line in the buffer view.
    /// If the provided line number exceeds the total number of lines in the buffer,
    /// it will be clamped to the maximum line count.
    ///
    /// # Parameters
    /// - `line`: The index of the line to set as the first visible line in the view.
    /// ```
    pub(crate) fn set_first_line(&mut self, line: usize) {
        info!("set first line: {line}");
        let line = line.min(self.buffer.line_count());
        self.scroll_offset.y = self.line_height * (line.saturating_sub(1) as f32);
    }

    pub(crate) const fn x_to_column(&self, x: f32) -> usize {
        (x / self.char_width).floor() as usize
    }

    pub(crate) const fn y_to_line(&self, y: f32) -> usize {
        (y / self.line_height).floor() as usize
    }

    pub(crate) const fn point_to_text_position(&self, point: Pos2) -> Position {
        Position {
            column: self.x_to_column(point.x),
            line: self.y_to_line(point.y),
        }
    }

    #[inline]
    pub(crate) fn gutter_width(&self) -> f32 {
        gutter::gutter_width(self.char_width, self.buffer.line_count())
    }

    #[inline]
    pub(crate) fn text_bounds(&self) -> Vec2 {
        Vec2::new(self.text_width(), self.text_height())
    }

    #[inline]
    pub(crate) fn text_width(&self) -> f32 {
        self.buffer.max_line_length() as f32 * self.char_width
    }

    #[inline]
    pub(crate) fn text_height(&self) -> f32 {
        self.line_height * self.buffer.line_count() as f32
    }

    pub(crate) fn get_row_range_for_rect(&self, rect: Rect) -> Range<usize> {
        let min_row = (rect.top() / self.line_height) as usize;
        let max_row = cmp::min(
            1 + (rect.bottom() / self.line_height) as usize,
            self.buffer.line_count(),
        );
        min_row..max_row
    }

    pub(crate) fn replace_selection(&mut self, text: &str) {
        self.delete_selection();
        for ch in text.chars() {
            if ch == '\r' || ch == '\x08' || ch == '\x7f' {
                continue;
            }
            self.buffer
                .insert_char(self.caret_position.line, self.caret_position.column, ch);
            if ch == '\n' {
                self.caret_position.line += 1;
                self.caret_position.column = 0;
            } else {
                self.caret_position.column += 1;
            }
        }
    }

    /// Delete the selection content if there is one.
    pub(crate) fn delete_selection(&mut self) {
        if let Some(selection) = self.selection.take() {
            self.buffer.delete_range(TextRange::from(&selection));
            self.caret_position = selection.start;
        }
    }

    pub(crate) fn copy(&self, ctx: &egui::Context) {
        info!("copy");
        if let Some(selection) = &self.selection {
            let mut text = String::with_capacity(5000);
            let start_line = selection.start.line;
            let end_line = selection.end.line;

            for line_idx in start_line..=end_line {
                let line_text = self.buffer.line_text(line_idx);
                let start_col = if line_idx == start_line { selection.start.column } else { 0 };
                let end_col = if line_idx == end_line { selection.end.column } else { line_text.len() };

                text.push_str(&line_text[start_col..end_col]);
                if line_idx < end_line {
                    text.push('\n');
                }
                if text.len() > MAX_COPY_SIZE {
                    // todo do a real egui thing
                    info!("Copy aborted: selection size {} exceeds limit {}", text.len(), MAX_COPY_SIZE);
                    return;
                }
            }
            debug!("About to copy: {text}");
            ctx.copy_text(text);
        }
    }

    pub(crate) fn cut(&mut self, ctx: &egui::Context) {
        self.copy(ctx);
        self.delete_selection();
    }

    pub(crate) fn go_to_prev_char(&mut self) {
        if self.caret_position.column > 0 {
            self.caret_position.column -= 1;
        } else if self.caret_position.line > 0 {
            self.caret_position.line -= 1;
            self.caret_position.column = self.buffer.line_length(self.caret_position.line).saturating_sub(1);
        }
    }

    pub(crate) fn go_to_next_char(&mut self) {
        if self.caret_position.column < self.buffer.line_length(self.caret_position.line) {
            self.caret_position.column += 1;
        } else if self.caret_position.line < self.buffer.line_count() - 1 {
            self.caret_position.line += 1;
            self.caret_position.column = 0;
        }
    }

    pub(crate) fn go_to_start_of_buffer(&mut self) {
        self.selection = None;
        self.caret_position = Position::ZERO;
    }

    pub(crate) fn go_to_start_of_line(&mut self) {
        self.selection = None;
        self.caret_position.column = 0;
    }

    pub(crate) fn go_to_end_of_line(&mut self) {
        self.selection = None;
        let current_line_length = self.buffer.line_text(self.caret_position.line);
        self.caret_position.column = current_line_length.len().saturating_sub(1);
    }

    pub(crate) fn go_to_end_of_buffer(&mut self) {
        self.selection = None;
        let current_line_length = self.buffer.line_text(self.buffer.line_count() - 1);
        self.caret_position.line = self.buffer.line_count().saturating_sub(1);
        self.caret_position.column = current_line_length.len().saturating_sub(1);
    }

    pub(crate) fn input_enter(&mut self) {
        self.buffer.insert_newline(
            self.caret_position.line,
            self.caret_position.column,
        );
        self.caret_position.line += 1;
        self.caret_position.column = 0;
    }

    pub(crate) fn input_backspace(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }

        if self.caret_position.column > 0 {
            let range = TextRange::new(
                self.caret_position.line,
                self.caret_position.column - 1,
                self.caret_position.line,
                self.caret_position.column,
            );
            self.buffer.delete_range(range);
            self.caret_position.column -= 1;
        } else if self.caret_position.line > 0 {
            let prev_line_idx = self.caret_position.line - 1;
            let prev_line_len = self.buffer.line_text(prev_line_idx).len();
            let range = TextRange::new(prev_line_idx, prev_line_len, self.caret_position.line, 0);
            self.buffer.delete_range(range);
            self.caret_position.line = prev_line_idx;
            self.caret_position.column = prev_line_len;
        }
    }

    pub(crate) fn input_delete(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }

        let line_len = self.buffer.line_text(self.caret_position.line).len();
        let line_count = self.buffer.line_count();
        if self.caret_position.column < line_len {
            let range = TextRange::new(
                self.caret_position.line,
                self.caret_position.column,
                self.caret_position.line,
                self.caret_position.column + 1,
            );
            self.buffer.delete_range(range);
        } else if self.caret_position.line + 1 < line_count {
            let range = TextRange::new(
                self.caret_position.line,
                line_len,
                self.caret_position.line + 1,
                0,
            );
            self.buffer.delete_range(range);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn create_test_textarea() -> TextAreaProperties {
        let (sender, _receiver) = mpsc::channel();
        TextAreaProperties::new(sender)
    }

    #[test]
    fn test_go_to_prev_char_within_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 2 };

        textarea.go_to_prev_char();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 1);
    }

    #[test]
    fn test_go_to_prev_char_to_previous_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.buffer.insert_newline(0, 3);
        textarea.buffer.insert_char(1, 0, 'x');
        textarea.caret_position = Position { line: 1, column: 0 };

        textarea.go_to_prev_char();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 2);
    }

    #[test]
    fn test_go_to_prev_char_at_start_of_buffer() {
        let mut textarea = create_test_textarea();
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.go_to_prev_char();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
    }

    #[test]
    fn test_go_to_next_char_within_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.go_to_next_char();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 1);
    }

    #[test]
    fn test_go_to_next_char_to_next_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.buffer.insert_newline(0, 3);
        textarea.buffer.insert_char(1, 0, 'x');
        textarea.caret_position = Position { line: 0, column: 3 };

        textarea.go_to_next_char();

        assert_eq!(textarea.caret_position.line, 1);
        assert_eq!(textarea.caret_position.column, 0);
    }

    #[test]
    fn test_go_to_next_char_at_end_of_buffer() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.caret_position = Position { line: 0, column: 2 };

        textarea.go_to_next_char();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 2);
    }

    #[test]
    fn test_handle_text_single_char() {
        let mut textarea = create_test_textarea();
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.replace_selection("a");

        assert_eq!(textarea.buffer.line_text(0), "a");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 1);
    }

    #[test]
    fn test_handle_text_with_newline() {
        let mut textarea = create_test_textarea();
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.replace_selection("ab\ncd");

        assert_eq!(textarea.buffer.line_text(0), "ab");
        assert_eq!(textarea.buffer.line_text(1), "cd");
        assert_eq!(textarea.caret_position.line, 1);
        assert_eq!(textarea.caret_position.column, 2);
    }

    #[test]
    fn test_handle_text_filters_control_chars() {
        let mut textarea = create_test_textarea();
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.replace_selection("a\rb\x08c\x7fd");

        assert_eq!(textarea.buffer.line_text(0), "abcd");
        assert_eq!(textarea.caret_position.column, 4);
    }

    #[test]
    fn test_delete_selection_with_selection() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.selection = Some(Selection {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 2 },
        });

        textarea.delete_selection();

        assert_eq!(textarea.buffer.line_text(0), "c");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_delete_selection_no_selection() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.selection = None;

        textarea.delete_selection();

        assert_eq!(textarea.buffer.line_text(0), "ab");
    }

    #[test]
    fn test_go_to_start_of_buffer() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_newline(0, 1);
        textarea.buffer.insert_char(1, 0, 'b');
        textarea.caret_position = Position { line: 1, column: 1 };
        textarea.selection = Some(Selection {
            start: Position { line: 0, column: 0 },
            end: Position { line: 1, column: 1 },
        });

        textarea.go_to_start_of_buffer();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_go_to_start_of_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 2 };

        textarea.go_to_start_of_line();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_go_to_end_of_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.go_to_end_of_line();

        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 2);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_go_to_end_of_buffer() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_newline(0, 1);
        textarea.buffer.insert_char(1, 0, 'b');
        textarea.buffer.insert_char(1, 1, 'c');
        textarea.caret_position = Position { line: 0, column: 0 };

        textarea.go_to_end_of_buffer();

        assert_eq!(textarea.caret_position.line, 1);
        assert_eq!(textarea.caret_position.column, 1);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_input_enter() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.caret_position = Position { line: 0, column: 1 };

        textarea.input_enter();

        assert_eq!(textarea.buffer.line_text(0), "a");
        assert_eq!(textarea.buffer.line_text(1), "b");
        assert_eq!(textarea.caret_position.line, 1);
        assert_eq!(textarea.caret_position.column, 0);
    }

    #[test]
    fn test_input_backspace_within_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 2 };

        textarea.input_backspace();

        assert_eq!(textarea.buffer.line_text(0), "ac");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 1);
    }

    #[test]
    fn test_input_backspace_at_line_start() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_newline(0, 2);
        textarea.buffer.insert_char(1, 0, 'c');
        textarea.caret_position = Position { line: 1, column: 0 };

        textarea.input_backspace();

        assert_eq!(textarea.buffer.line_text(0), "abc");
        assert_eq!(textarea.buffer.line_count(), 1);
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 2);
    }

    #[test]
    fn test_input_backspace_with_selection() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 2 };
        textarea.selection = Some(Selection {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 2 },
        });

        textarea.input_backspace();

        assert_eq!(textarea.buffer.line_text(0), "c");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
        assert!(textarea.selection.is_none());
    }

    #[test]
    fn test_input_delete_within_line() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 1 };

        textarea.input_delete();

        assert_eq!(textarea.buffer.line_text(0), "ac");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 1);
    }

    #[test]
    fn test_input_delete_at_line_end() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_newline(0, 2);
        textarea.buffer.insert_char(1, 0, 'c');
        textarea.caret_position = Position { line: 0, column: 2 };

        textarea.input_delete();

        assert_eq!(textarea.buffer.line_text(0), "abc");
        assert_eq!(textarea.buffer.line_count(), 1);
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 2);
    }

    #[test]
    fn test_input_delete_with_selection() {
        let mut textarea = create_test_textarea();
        textarea.buffer.insert_char(0, 0, 'a');
        textarea.buffer.insert_char(0, 1, 'b');
        textarea.buffer.insert_char(0, 2, 'c');
        textarea.caret_position = Position { line: 0, column: 0 };
        textarea.selection = Some(Selection {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 2 },
        });

        textarea.input_delete();

        assert_eq!(textarea.buffer.line_text(0), "c");
        assert_eq!(textarea.caret_position.line, 0);
        assert_eq!(textarea.caret_position.column, 0);
        assert!(textarea.selection.is_none());
    }
}