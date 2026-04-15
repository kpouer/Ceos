use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::highlight::highlight::Highlight;
use eframe::emath::{Pos2, Rect};
use egui::{TextBuffer, Ui};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct HighlightPainter;

impl Renderer for HighlightPainter {
    fn paint_line(
        &self,
        ui: &mut Ui,
        _theme: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
        _has_focus: bool,
    ) {
        let highlight_manager = &textarea_properties.highlight_manager;

        let text = textarea_properties.buffer.line_text(line);
        for highlight in highlight_manager.iter() {
            self.paint_highlight(ui, drawing_pos, &text, highlight, textarea_properties);
        }
    }
}

impl HighlightPainter {
    pub(crate) fn paint_highlight(
        &self,
        ui: &mut Ui,
        drawing_pos: Pos2,
        text: &str,
        highlight: &Highlight,
        textarea_properties: &TextAreaProperties,
    ) {
        let pattern: Cow<'_, str> = if highlight.case_insensitive {
            Cow::Owned(highlight.text.to_lowercase())
        } else {
            Cow::Borrowed(&highlight.text)
        };

        let search_text: Cow<'_, str> = if highlight.case_insensitive {
            Cow::Owned(text.to_lowercase())
        } else {
            Cow::Borrowed(text)
        };

        let mut start_index = 0;
        while let Some(byte_index) = search_text[start_index..].find(pattern.as_str()) {
            let actual_byte_index = start_index + byte_index;
            let char_index = search_text[..actual_byte_index].chars().count();
            let char_count = pattern.chars().count();

            let rect = Rect::from_min_max(
                Pos2::new(
                    drawing_pos.x + char_index as f32 * textarea_properties.char_width,
                    drawing_pos.y,
                ),
                Pos2::new(
                    drawing_pos.x
                        + (char_index + char_count) as f32 * textarea_properties.char_width,
                    drawing_pos.y + textarea_properties.line_height,
                ),
            );

            if ui.clip_rect().intersects(rect) {
                ui.painter().rect_filled(rect, 0.0, highlight.color);
            }

            start_index = actual_byte_index + pattern.len();
        }
    }
}
