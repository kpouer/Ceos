use eframe::emath::Pos2;
use eframe::epaint::FontId;
use egui::Ui;

use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;

pub(crate) mod caret_renderer;
pub(crate) mod renderer_manager;
pub(crate) mod selection_renderer;
pub(crate) mod text_renderer;

pub(crate) trait Renderer {
    fn before_frame(&mut self) {}

    fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
    );

    fn set_font_id(&mut self, _font_id: FontId) {}
}
