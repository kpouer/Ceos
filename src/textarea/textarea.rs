use std::cmp;
use eframe::epaint::FontId;
use log::info;

use crate::ceos::command::Command;
use crate::textarea::buffer::Buffer;
use crate::textarea::buffer_properties::BufferProperties;
use crate::textarea::renderer::Renderer;
use crate::textarea::renderer::text_renderer::TextRenderer;

const DEFAULT_LINE_HEIGHT: f32 = 16.0;

pub(crate) struct TextArea {
    buffer: Buffer,
    buffer_properties: BufferProperties,
    renderers: Vec<Box<dyn Renderer>>,
    line_height: f32,
    font_id: FontId,
}

impl Default for TextArea {
    fn default() -> Self {
        let font_id = egui::FontId::new(DEFAULT_LINE_HEIGHT, egui::FontFamily::Monospace);
        let renderers: Vec<Box<dyn Renderer>> = vec![
            Box::new(TextRenderer::from(font_id.clone())),
        ];
        Self {
            buffer: Default::default(),
            buffer_properties: Default::default(),
            renderers,
            line_height: DEFAULT_LINE_HEIGHT,
            font_id,
        }
    }
}

impl TextArea {
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

    pub(crate) fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub(crate) fn scroll_down(&mut self) {
        info!("scrolling down");
        let firstline = self.buffer_properties.first_line();
        let lastline = self.buffer.content().len();
        if lastline - firstline <= 1 {
            return;
        }
        self.buffer_properties.set_first_line(firstline + 1);
    }

    pub(crate) fn scroll_up(&mut self) {
        info!("scrolling up");
        let firstline = self.buffer_properties.first_line();
        if firstline == 0 {
            return;
        }
        self.buffer_properties.set_first_line(firstline - 1);
    }

    pub(crate) fn scroll_left(&mut self) {
        info!("scrolling left");
        let horizontal_offset = self.horizontal_offset();
        if horizontal_offset == 0 {
            return;
        }
        self.buffer_properties
            .set_horizontal_offset(horizontal_offset - 1);
    }

    pub(crate) fn scroll_right(&mut self) {
        info!("scrolling right");
        let horizontal_offset = self.horizontal_offset();
        self.buffer_properties
            .set_horizontal_offset(horizontal_offset + 1);
    }

    pub(crate) fn horizontal_offset(&self) -> usize {
        self.buffer_properties.horizontal_offset()
    }
}

impl From<Buffer> for TextArea {
    fn from(buffer: Buffer) -> Self {
        Self {
            buffer,
            ..Default::default()
        }
    }
}

impl TextArea {
    pub(crate) fn show(&self, ui: &mut egui::Ui, filter_renderer: &Option<Box<dyn Command>>) {
        let rect = ui.max_rect();
        let max_screen_lines = (rect.height() / self.line_height).floor() as usize;
        let firstline = self.buffer_properties.first_line();
        let lastline = self.buffer.content().len();
        let mut pos = rect.left_top();
        for screenline in 0..cmp::min(max_screen_lines, lastline) {
            if let Some(filter_renderer) = filter_renderer {
                filter_renderer.paint_line(ui, self, screenline + firstline, pos);
            }
            self.renderers
                .iter()
                .for_each(|r| r.paint_line(ui, self, screenline + firstline, pos));
            pos.y += self.line_height;
        }
    }
}
