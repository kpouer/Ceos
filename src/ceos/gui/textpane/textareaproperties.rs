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
}

impl Default for TextAreaProperties {
    fn default() -> Self {
        let font_id = FontId::new(DEFAULT_LINE_HEIGHT, egui::FontFamily::Monospace);
        let mut renderer_manager = RendererManager::default();
        renderer_manager.add_renderer(TEXT_LAYER, Box::new(TextRenderer::new(font_id.clone())));
        renderer_manager.add_renderer(SELECTION_LAYER, Box::new(SelectionRenderer {}));
        renderer_manager.add_renderer(CARET_LAYER, Box::new(CaretRenderer::default()));
        Self {
            buffer: Default::default(),
            renderer_manager,
            line_height: DEFAULT_LINE_HEIGHT,
            font_id,
            char_width: 0.0,
            caret_position: Position::default(),
            selection: None,
        }
    }
}

impl TextAreaProperties {
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

    pub(crate) fn x_to_column(&self, x: f32) -> usize {
        (x / self.char_width).floor() as usize
    }

    pub(crate) fn y_to_line(&self, y: f32) -> usize {
        (y / self.line_height).floor() as usize
    }

    pub(crate) fn point_to_text_position(&self, point: Pos2) -> (usize, usize) {
        (self.x_to_column(point.x), self.y_to_line(point.y))
    }

    pub(crate) fn gutter_width(&self) -> f32 {
        gutter::gutter_width(self.char_width, self.buffer.line_count())
    }

    pub(crate) fn text_bounds(&self) -> Vec2 {
        Vec2::new(self.text_width(), self.text_height())
    }

    pub(crate) fn text_width(&self) -> f32 {
        self.buffer.max_line_length() as f32 * self.char_width
    }

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

impl From<Buffer> for TextAreaProperties {
    fn from(buffer: Buffer) -> Self {
        Self {
            buffer,
            ..Default::default()
        }
    }
}
