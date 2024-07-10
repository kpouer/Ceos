use eframe::emath::Pos2;
use egui::Ui;

use crate::textarea::renderer::Renderer;
use crate::textarea::textarea::TextArea;

#[derive(Default)]
pub(crate) struct TextRenderer;

impl Renderer for TextRenderer {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextArea, line: usize, pos: Pos2) {
        let painter = ui.painter();
        let text = &textarea.buffer().content()[line].content();
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
        painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::new(textarea.line_height(), egui::FontFamily::Monospace),
            ui.visuals().text_color(),
        );
    }
}
