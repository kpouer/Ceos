use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::Pos2;
use eframe::epaint::StrokeKind;
use egui::Rect;

pub(crate) struct SelectionRenderer;

impl Renderer for SelectionRenderer {
    fn paint_line(
        &self,
        ui: &mut egui::Ui,
        _: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
    ) {
        if let Some(selection) = &textarea_properties.selection {
            if selection.line == line {
                let start_x =
                    drawing_pos.x + selection.start_column as f32 * textarea_properties.char_width;
                let end_x =
                    drawing_pos.x + selection.end_column as f32 * textarea_properties.char_width;
                let rect = Rect::from([
                    Pos2::new(start_x, drawing_pos.y),
                    Pos2::new(end_x, drawing_pos.y + textarea_properties.line_height),
                ]);
                ui.painter().rect(
                    rect,
                    0.0,
                    ui.style().visuals.selection.bg_fill,
                    ui.style().visuals.selection.stroke,
                    StrokeKind::Inside,
                );
            }
        }
    }
}
