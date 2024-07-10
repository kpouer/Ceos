use eframe::emath::Pos2;
use egui::Ui;

use crate::textarea::textarea::TextArea;

pub(crate) mod text_renderer;

pub(crate) trait Renderer {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextArea, line: usize, pos: Pos2);
}
