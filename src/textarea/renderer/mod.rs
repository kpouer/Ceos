use eframe::emath::Pos2;
use egui::Ui;

use crate::textarea::textareaproperties::TextAreaProperties;

pub(crate) mod text_renderer;

pub(crate) trait Renderer {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextAreaProperties, line: usize, virtual_pos: Pos2, drawing_pos: Pos2);
}
