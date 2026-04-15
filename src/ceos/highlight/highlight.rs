use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::{Pos2, Rect};
use egui::{Color32, Ui};

#[derive(Debug, Clone)]
pub(crate) struct Highlight {
    pub(crate) text: String,
    pub(crate) case_insensitive: bool,
    pub(crate) color: Color32,
}

impl Highlight {
    pub(crate) const fn new(text: String, case_insensitive: bool, color: Color32) -> Self {
        Self {
            text,
            case_insensitive,
            color,
        }
    }
}

impl Renderer for Highlight {
    fn paint_line(
        &self,
        ui: &mut Ui,
        _theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
        _has_focus: bool,
    ) {
        let text = textarea.buffer.line_text(line);
        let search_text = if self.case_insensitive {
            text.to_lowercase()
        } else {
            text.to_string()
        };
        let pattern = if self.case_insensitive {
            self.text.to_lowercase()
        } else {
            self.text.clone()
        };

        if pattern.is_empty() {
            return;
        }

        let mut start_index = 0;
        while let Some(byte_index) = search_text[start_index..].find(&pattern) {
            let actual_byte_index = start_index + byte_index;
            let char_index = search_text[..actual_byte_index].chars().count();
            let char_count = pattern.chars().count();

            let rect = Rect::from_min_max(
                Pos2::new(
                    drawing_pos.x + char_index as f32 * textarea.char_width,
                    drawing_pos.y,
                ),
                Pos2::new(
                    drawing_pos.x + (char_index + char_count) as f32 * textarea.char_width,
                    drawing_pos.y + textarea.line_height,
                ),
            );

            ui.painter().rect_filled(rect, 0.0, self.color);

            start_index = actual_byte_index + pattern.len();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_highlight_rendering_multi_byte() {
        let text = "Hello, 🌍 world!";
        let pattern = "world";
        let search_text = text.to_string();

        let byte_index = search_text.find(pattern).unwrap();
        let char_index = search_text[..byte_index].chars().count();
        let char_count = pattern.chars().count();

        // H(0) e(1) l(2) l(3) o(4) ,(5)  (6) 🌍(7)  (8) w(9)
        // Correct is 9.
        assert_eq!(char_index, 9);
        assert_eq!(char_count, 5);
    }
}
