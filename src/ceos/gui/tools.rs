use eframe::epaint::{Color32, FontId};
use egui::Ui;

pub(crate) fn char_width(font_id: FontId, ui: &Ui) -> f32 {
    let painter = ui.painter();
    let layout = painter.layout("A".to_string(), font_id, Color32::RED, 0f32);
    layout.size().x
}
