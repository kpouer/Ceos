use crate::ceos::highlight::highlight::Highlight;
use eframe::epaint::Color32;
use log::info;
use std::slice::Iter;

const COLORS: [Color32; 11] = [
    Color32::from_rgb(0x99, 0xff, 0xcc),
    Color32::from_rgb(0x66, 0x66, 0xff),
    Color32::from_rgb(0xff, 0x66, 0x66),
    Color32::from_rgb(0xff, 0xcc, 0x66),
    Color32::from_rgb(0xcc, 0xff, 0x66),
    Color32::from_rgb(0xff, 0x33, 0x99),
    Color32::from_rgb(0xff, 0x33, 0x00),
    Color32::from_rgb(0x66, 0xff, 0x00),
    Color32::from_rgb(0x99, 0x00, 0x99),
    Color32::from_rgb(0x99, 0x99, 0x00),
    Color32::from_rgb(0x00, 0x99, 0x66),
];

#[derive(Debug, Default)]
pub(crate) struct HighlightManager {
    color_index: usize,
    highlights: Vec<Highlight>,
}

impl HighlightManager {
    pub(crate) fn add(&mut self, highlight: Highlight) {
        if self.has_highlight_with_text(&highlight.text) {
            return;
        }
        info!("add({highlight:?})");
        self.highlights.push(highlight);
    }

    pub(crate) fn create_highlight(&mut self, text: String) -> Highlight {
        info!("add_highlight {text}");
        let color = COLORS[self.color_index];
        self.color_index += 1 % COLORS.len();
        Highlight::new(text, false, color)
    }

    pub(crate) fn remove(&mut self, index: usize) {
        if index < self.highlights.len() {
            self.highlights.remove(index);
        }
    }

    #[inline]
    fn has_highlight_with_text(&self, text: &str) -> bool {
        self.highlights
            .iter()
            .any(|highlight| highlight.text == text)
    }

    #[inline]
    pub(crate) const fn len(&self) -> usize {
        self.highlights.len()
    }

    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, Highlight> {
        self.highlights.iter()
    }
}
