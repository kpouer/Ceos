use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::gui::textpane::gutter;
use crate::ceos::gui::textpane::position::Position;
use crate::ceos::gui::textpane::renderer::caret_renderer::CaretRenderer;
use crate::ceos::gui::textpane::renderer::renderer_manager::{
    CARET_LAYER, RendererManager, SELECTION_LAYER, TEXT_LAYER,
};
use crate::ceos::gui::textpane::renderer::selection_renderer::SelectionRenderer;
use crate::ceos::gui::textpane::renderer::text_renderer::TextRenderer;
use crate::ceos::gui::textpane::selection::Selection;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::FontId;
use log::info;
use std::cmp;
use std::ops::Range;
use std::sync::mpsc::Sender;
use crate::ceos::gui::textpane::interaction_mode::InteractionMode;
use crate::event::Event;

pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 16.0;

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
        renderer_manager.add_renderer(CARET_LAYER, Box::new(CaretRenderer::default()));
        Self {
            buffer: Buffer::new(sender),
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
        self.scroll_offset.y = self.line_height * ((line - 1) as f32);
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
}
