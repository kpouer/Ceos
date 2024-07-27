use crate::ceos::gui::textarea::renderer::Renderer;
use crate::ceos::gui::textarea::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::syntax::tokenizer::Tokenizer;
use eframe::emath::Pos2;
use egui::{FontId, Ui};

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
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        virtual_pos: Pos2,
        mut drawing_pos: Pos2,
    ) {
        let text = textarea.buffer().line_text(line);
        if text.trim().is_empty() {
            return;
        }
        let text = Self::get_text_to_render(textarea, virtual_pos.x, text);
        let painter = ui.painter();
        let mut tokenizer = Tokenizer::new(text);
        tokenizer.merge_tokens();
        let initial_offset = drawing_pos.x;
        tokenizer.tokens.into_iter().for_each(|chunk| {
            drawing_pos.x = initial_offset + chunk.start() as f32 * textarea.char_width();
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
        // painter.text(
        //     drawing_pos,
        //     egui::Align2::LEFT_TOP,
        //     text,
        //     self.font_id.clone(),
        //     ui.visuals().text_color(),
        // );
    }

    fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id;
    }
}

impl TextRenderer {
    fn get_text_to_render<'a>(
        textarea: &TextAreaProperties,
        x_offset: f32,
        text: &'a str,
    ) -> &'a str {
        let column = Self::get_start_column(textarea, x_offset, text);
        &text[column..]
    }

    ///because some chars are 2 bytes
    fn get_start_column(textarea: &TextAreaProperties, x_offset: f32, text: &str) -> usize {
        let start_column = textarea.offset_x_to_column(x_offset);
        let column = if start_column == 0 {
            0
        } else {
            text.char_indices()
                .map(|(i, _)| i)
                .nth(start_column)
                .unwrap_or(0)
        };
        column
    }
}
