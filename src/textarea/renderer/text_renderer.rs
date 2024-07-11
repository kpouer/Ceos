use eframe::emath::Pos2;
use egui::{FontId, Ui};

use crate::textarea::renderer::Renderer;
use crate::textarea::textarea::TextArea;

pub(crate) struct TextRenderer {
    font_id: FontId,
}

impl From<FontId> for TextRenderer {
    fn from(font_id: FontId) -> Self {
        Self { font_id }
    }
}

impl Renderer for TextRenderer {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextArea, line: usize, pos: Pos2) {
        let text = textarea.buffer().content()[line].content();
        if text.is_empty() {
            return;
        }
        let horizontal_offset = textarea.horizontal_offset();
        if horizontal_offset >= text.len() {
            return;
        }
        //because some chars are 2 bytes
        let start = text
            .char_indices()
            .map(|(i, _)| i)
            .nth(horizontal_offset)
            .unwrap();

        let text = &text[start..];
        let painter = ui.painter();
        painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            self.font_id.clone(),
            ui.visuals().text_color(),
        );
    }
}
