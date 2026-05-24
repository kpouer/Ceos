use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use buffer_core::selection::Selection;
use eframe::emath::Pos2;
use egui::Rect;

#[derive(Debug)]
pub(crate) struct SelectionRenderer;

impl Renderer for SelectionRenderer {
    fn paint_line(
        &self,
        ui: &mut egui::Ui,
        _theme: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        line_text: &str,
        drawing_pos: Pos2,
        _has_focus: bool,
    ) {
        let Some(selection) = &textarea_properties.selection else {
            return;
        };
        let (Some(start_x), Some(end_x)) = Self::get_start_stop(
            &selection,
            line,
            line_text,
            drawing_pos.x,
            textarea_properties.char_width,
        ) else {
            return;
        };

        let rect = Rect::from([
            Pos2::new(start_x, drawing_pos.y),
            Pos2::new(end_x, drawing_pos.y + textarea_properties.line_height),
        ]);
        ui.painter()
            .rect_filled(rect, 0.0, ui.style().visuals.selection.bg_fill);
    }
}

impl SelectionRenderer {
    const fn get_start_stop(
        selection: &Selection,
        line: usize,
        line_text: &str,
        drawing_pos_x: f32,
        char_width: f32,
    ) -> (Option<f32>, Option<f32>) {
        (
            Self::start_x(selection, line, drawing_pos_x, char_width),
            Self::end_x(selection, line, drawing_pos_x, char_width, line_text),
        )
    }

    const fn start_x(
        selection: &Selection,
        line: usize,
        drawing_pos_x: f32,
        char_width: f32,
    ) -> Option<f32> {
        if selection.start.line < line {
            Some(drawing_pos_x)
        } else if selection.start.line == line {
            Some(drawing_pos_x + selection.start.column as f32 * char_width)
        } else {
            None
        }
    }

    const fn end_x(
        selection: &Selection,
        line: usize,
        drawing_pos_x: f32,
        char_width: f32,
        line_text: &str,
    ) -> Option<f32> {
        let end_column = if selection.end.line == line {
            selection.end.column
        } else if selection.end.line > line {
            line_text.len()
        } else {
            return None;
        };
        Some(drawing_pos_x + end_column as f32 * char_width)
    }
}
