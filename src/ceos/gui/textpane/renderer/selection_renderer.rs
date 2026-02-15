use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
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
        drawing_pos: Pos2,
        _has_focus: bool,
    ) {
        if let Some(selection) = &textarea_properties.selection
        {
            let start_column = if selection.start.line < line {
                0
            } else if selection.start.line == line {
                selection.start.column
            } else {
                return;
            } as f32;
            let end_column = if selection.end.line == line {
                selection.end.column
            } else if selection.end.line > line {
                textarea_properties.buffer.line_text(line).len()
            } else {
                return;
            } as f32;
            let start_x = drawing_pos.x + start_column * textarea_properties.char_width;
            let end_x = drawing_pos.x + end_column * textarea_properties.char_width;
            let rect = Rect::from([
                Pos2::new(start_x, drawing_pos.y),
                Pos2::new(end_x, drawing_pos.y + textarea_properties.line_height),
            ]);
            ui.painter().rect_filled(
                rect,
                0.0,
                ui.style().visuals.selection.bg_fill,
            );
        }
    }
}
