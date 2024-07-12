use eframe::emath::Pos2;
use egui::{FontId, Ui};

use crate::textarea::renderer::Renderer;
use crate::textarea::textareaproperties::TextAreaProperties;

pub(crate) struct TextRenderer {
    font_id: FontId,
}

impl From<FontId> for TextRenderer {
    fn from(font_id: FontId) -> Self {
        Self { font_id }
    }
}

impl Renderer for TextRenderer {
    fn paint_line(
        &self,
        ui: &mut Ui,
        textarea: &TextAreaProperties,
        line: usize,
        virtual_pos: Pos2,
        drawing_pos: Pos2,
    ) {
        let text = textarea.buffer().line_text(line);
        if text.trim().is_empty() {
            return;
        }
        //because some chars are 2 bytes
        let start_column = textarea.offset_x_to_column(virtual_pos.x);
        let column = text
            .char_indices()
            .map(|(i, _)| i)
            .nth(start_column)
            .unwrap_or(0);
        let text = &text[column..];
        let painter = ui.painter();
        painter.text(
            drawing_pos,
            egui::Align2::LEFT_TOP,
            text,
            self.font_id.clone(),
            ui.visuals().text_color(),
        );
    }
}
