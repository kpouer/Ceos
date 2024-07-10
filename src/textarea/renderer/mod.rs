use crate::textarea::textarea::TextArea;
use eframe::emath::Pos2;
use egui::Ui;

pub(crate) mod text_renderer;

pub(crate) trait Renderer {
    fn paint_line(&self, ui: &mut Ui, textarea: &TextArea, line: usize, pos: Pos2);
}
