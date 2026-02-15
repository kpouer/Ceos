use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::Pos2;
use egui::Rect;
use egui::text_selection::visuals::paint_text_cursor;

#[derive(Debug, Default)]
pub(crate) struct CaretRenderer;

impl Renderer for CaretRenderer {
    fn paint_line(
        &self,
        ui: &mut egui::Ui,
        _: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
        has_focus: bool,
    ) {
        if !has_focus || textarea_properties.caret_position.line != line {
            return;
        }

        let now = ui.ctx().input(|i| i.time);
        let x = drawing_pos.x
            + textarea_properties.caret_position.column as f32 * textarea_properties.char_width;
        let rect = Rect::from([
            Pos2::new(x, drawing_pos.y),
            Pos2::new(x, drawing_pos.y + textarea_properties.line_height),
        ]);
        paint_text_cursor(ui, ui.painter(), rect, now);
    }
}
