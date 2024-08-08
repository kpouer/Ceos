use eframe::emath::Pos2;
use eframe::epaint::FontId;
use egui::Ui;

use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;

pub(crate) mod text_renderer;

pub(crate) trait Renderer {
    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea: &TextAreaProperties,
        line: usize,
        virtual_pos: Pos2,
        drawing_pos: Pos2,
    );

    fn set_font_id(&mut self, _font_id: FontId) {}
}
