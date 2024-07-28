use crate::ceos::buffer::Buffer;
use crate::ceos::gui::textarea::renderer::text_renderer::TextRenderer;
use crate::ceos::gui::textarea::renderer::Renderer;
use crate::ceos::gui::widget::gutter;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::FontId;
use log::info;
use std::cmp;
use std::ops::Range;

pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 16.0;

pub(crate) struct TextAreaProperties {
    pub(crate) buffer: Buffer,
    pub(crate) renderers: Vec<Box<dyn Renderer>>,
    pub(crate) line_height: f32,
    pub(crate) font_id: FontId,
    pub(crate) char_width: f32,
    pub(crate) scroll_offset: Vec2,
}

impl Default for TextAreaProperties {
    fn default() -> Self {
        let font_id = egui::FontId::new(DEFAULT_LINE_HEIGHT, egui::FontFamily::Monospace);
        let renderers: Vec<Box<dyn Renderer>> = vec![Box::new(TextRenderer::from(font_id.clone()))];
        Self {
            buffer: Default::default(),
            renderers,
            line_height: DEFAULT_LINE_HEIGHT,
            font_id,
            char_width: 0.0,
            scroll_offset: Vec2::ZERO,
        }
    }
}

impl TextAreaProperties {
    pub(crate) fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id.clone();
        self.renderers
            .iter_mut()
            .for_each(|r| r.set_font_id(font_id.clone()));
        self.char_width = 0.0;
        self.line_height = font_id.size;
    }

    pub(crate) fn set_buffer(&mut self, buffer: Buffer) {
        info!(
            "set buffer: {}, line count: {}",
            buffer.path,
            buffer.line_count()
        );
        self.buffer = buffer
    }

    pub(crate) fn offset_x_to_column(&self, x: f32) -> usize {
        (x / self.char_width).floor() as usize
    }

    pub(crate) fn y_to_line(&self, y: f32) -> usize {
        (y / self.line_height).floor() as usize
    }

    pub(crate) fn point_to_text_position(&self, point: Pos2) -> (usize, usize) {
        (self.offset_x_to_column(point.x), self.y_to_line(point.y))
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
