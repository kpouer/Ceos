use eframe::emath::Pos2;
use eframe::epaint::{Color32, FontId};
use egui::Ui;

pub(crate) fn char_width(font_id: FontId, ui: &Ui) -> f32 {
    let painter = ui.painter();
    let layout = painter.layout("A".to_string(), font_id, Color32::RED, 0f32);
    layout.size().x
}

pub(crate) fn is_off_screen(ui: &Ui, pos: Pos2) -> bool {
    if pos.y < 0.0 {
        return true;
    }
    
    if pos.y > ui.max_rect().height() {
        return true;
    }
    false
}