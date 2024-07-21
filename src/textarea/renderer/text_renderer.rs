use crate::syntax::tokenizer::Tokenizer;
use crate::textarea::renderer::Renderer;
use crate::textarea::textareaproperties::TextAreaProperties;
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
        let tokenizer = Tokenizer::new(text);
        let chunks = tokenizer.tokenize();
        let initial_offset = drawing_pos.x;
        chunks.into_iter().for_each(|chunk| {
            drawing_pos.x = initial_offset + chunk.start() as f32 * textarea.char_width();
            painter.text(
                drawing_pos,
                egui::Align2::LEFT_TOP,
                chunk.as_str(),
                self.font_id.clone(),
                chunk.color().unwrap_or(ui.visuals().text_color()),
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
