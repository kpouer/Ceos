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
        if let Some(selection) = &textarea_properties.selection {
            let Some(start_column) = Self::start_column(&selection, line) else {
                return;
            };
            let Some(end_column) = Self::end_column(&selection, line, line_text) else {
                return;
            };
            let start_x = drawing_pos.x + start_column * textarea_properties.char_width;
            let end_x = drawing_pos.x + end_column * textarea_properties.char_width;
            let rect = Rect::from([
                Pos2::new(start_x, drawing_pos.y),
                Pos2::new(end_x, drawing_pos.y + textarea_properties.line_height),
            ]);
            ui.painter()
                .rect_filled(rect, 0.0, ui.style().visuals.selection.bg_fill);
        }
    }
}

impl SelectionRenderer {
    fn start_column(selection: &Selection, line: usize) -> Option<f32> {
        if selection.start.line < line {
            Some(0.0)
        } else if selection.start.line == line {
            Some(selection.start.column as f32)
        } else {
            None
        }
    }

    fn end_column(selection: &Selection, line: usize, line_text: &str) -> Option<f32> {
        if selection.end.line == line {
            Some(selection.end.column as f32)
        } else if selection.end.line > line {
            Some(line_text.len() as f32)
        } else {
            None
        }
    }
}
