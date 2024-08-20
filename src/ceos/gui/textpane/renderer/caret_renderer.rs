use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::Pos2;
use egui::text_selection::visuals::paint_text_cursor;
use egui::Rect;
use std::time::Duration;

const PERIOD: Duration = Duration::from_millis(500);

pub(crate) struct CaretRenderer {
    last_change: f64,
}

impl Default for CaretRenderer {
    fn default() -> Self {
        Self { last_change: 0.0 }
    }
}

impl Renderer for CaretRenderer {
    fn paint_line(
        &self,
        ui: &mut egui::Ui,
        _: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: egui::Pos2,
    ) {
        let now = ui.ctx().input(|i| i.time);
        let time_since_last_edit = now - self.last_change;
        if textarea_properties.caret_position.line == line {
            let x = drawing_pos.x
                + textarea_properties.caret_position.column as f32 * textarea_properties.char_width;
            let rect = Rect::from([
                Pos2::new(x, drawing_pos.y),
                Pos2::new(x, drawing_pos.y + textarea_properties.line_height),
            ]);
            paint_text_cursor(ui, ui.painter(), rect, time_since_last_edit);
        }
    }
}
