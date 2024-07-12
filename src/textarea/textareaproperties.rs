use crate::ceos::gui::widget::gutter;
use crate::textarea::buffer::Buffer;
use crate::textarea::renderer::text_renderer::TextRenderer;
use crate::textarea::renderer::Renderer;
use eframe::emath::{Rect, Vec2};
use eframe::epaint::FontId;
use std::cmp;
use std::ops::Range;

pub(crate) const DEFAULT_LINE_HEIGHT: f32 = 16.0;

pub(crate) struct TextAreaProperties {
    buffer: Buffer,
    renderers: Vec<Box<dyn Renderer>>,
    line_height: f32,
    font_id: FontId,
    char_width: f32,
    scroll_offset: Vec2,
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
    pub(crate) fn set_scroll_offset(&mut self, scroll_offset: Vec2) {
        self.scroll_offset = scroll_offset
    }

    pub(crate) fn char_width(&self) -> f32 {
        self.char_width
    }

    pub(crate) fn set_char_width(&mut self, char_width: f32) {
        self.char_width = char_width;
    }

    pub(crate) fn font_id(&self) -> &FontId {
        &self.font_id
    }

    pub(crate) fn set_buffer(&mut self, buffer: Buffer) {
        self.buffer = buffer
    }

    pub(crate) fn line_height(&self) -> f32 {
        self.line_height
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(crate) fn offset_x_to_column(&self, x: f32) -> usize {
        (x / self.char_width()).floor() as usize
    }

    pub(crate) fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn renderers(&self) -> &Vec<Box<dyn Renderer>> {
        &self.renderers
    }

    pub fn scroll_offset(&self) -> &Vec2 {
        &self.scroll_offset
    }

    pub(crate) fn gutter_width(&self) -> f32 {
        gutter::gutter_width(self.char_width(), &self.buffer())
    }

    pub(crate) fn bounds(&self) -> Vec2 {
        Vec2::new(self.gutter_width() + self.text_width(), self.text_height())
    }

    pub(crate) fn text_bounds(&self) -> Vec2 {
        Vec2::new(self.text_width(), self.text_height())
    }

    pub(crate) fn text_width(&self) -> f32 {
        self.buffer().max_line_length() as f32 * self.char_width()
    }

    pub(crate) fn text_height(&self) -> f32 {
        self.line_height() * self.buffer.line_count() as f32
    }

    pub(crate) fn get_row_range_for_rect(&self, rect: Rect) -> Range<usize> {
        let min_row = (rect.top() / self.line_height()) as usize;
        let max_row = cmp::min(
            (rect.bottom() / self.line_height()) as usize,
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
