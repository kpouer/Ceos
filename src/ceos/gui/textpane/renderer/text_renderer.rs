use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::syntax::tokenizer::Tokenizer;
use eframe::emath::Pos2;
use egui::{FontId, Ui};

#[derive(Debug)]
pub(crate) struct TextRenderer {
    font_id: FontId,
}

impl TextRenderer {
    pub(crate) fn new(font_id: FontId) -> Self {
        Self { font_id }
    }
}

impl Renderer for TextRenderer {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        mut drawing_pos: Pos2,
    ) {
        let text = textarea.buffer.line_text(line);
        if text.trim().is_empty() {
            return;
        }
        let painter = ui.painter();
        let mut tokenizer = Tokenizer::new(text);
        tokenizer.merge_tokens();
        let initial_offset = drawing_pos.x;
        tokenizer.tokens.into_iter().for_each(|chunk| {
            drawing_pos.x = initial_offset + chunk.start() as f32 * textarea.char_width;
            let color = chunk
                .token
                .as_ref()
                .map(|token| theme.color(token))
                .unwrap_or(theme.text);
            painter.text(
                drawing_pos,
                egui::Align2::LEFT_TOP,
                chunk.as_str(),
                self.font_id.clone(),
                color,
            );
        });
    }

    fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id;
    }
}
