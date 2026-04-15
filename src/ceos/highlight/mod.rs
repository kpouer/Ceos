pub(crate) mod highlight;
pub(crate) mod highlight_painter;
pub(crate) mod manager;

pub(crate) fn deterministic_color(text: &str) -> egui::Color32 {
    let mut hash: u32 = 0;
    for b in text.bytes() {
        hash = hash.wrapping_add(b as u32).wrapping_mul(0x85ebca6b);
    }
    let r = (hash & 0xFF) as u8;
    let g = ((hash >> 8) & 0xFF) as u8;
    let b = ((hash >> 16) & 0xFF) as u8;
    egui::Color32::from_rgb(r, g, b)
}
