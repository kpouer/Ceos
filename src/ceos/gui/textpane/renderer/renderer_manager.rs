use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::Pos2;
use eframe::epaint::FontId;
use egui::Ui;

pub(crate) const BASE_LAYER: u8 = 0;
pub(crate) const SELECTION_LAYER: u8 = 30;
pub(crate) const HIGHLIGHT_LAYER: u8 = 50;
pub(crate) const TEXT_LAYER: u8 = 100;
pub(crate) const CARET_LAYER: u8 = 150;

#[derive(Default, Debug)]
pub(crate) struct RendererManager {
    pub(crate) renderers: Vec<RendererEntry>,
}

impl RendererManager {
    pub(crate) fn add_renderer(&mut self, layer: u8, renderer: Box<dyn Renderer>) {
        let entry = RendererEntry { layer, renderer };
        let index = self.renderers.partition_point(|e| e.layer <= entry.layer);
        self.renderers.insert(index, entry);
    }

    pub(crate) fn before_frame(&mut self) {
        self.renderers
            .iter_mut()
            .for_each(|r| r.renderer.before_frame());
    }

    pub(crate) fn set_font_id(&mut self, font_id: FontId) {
        self.renderers
            .iter_mut()
            .for_each(|r| r.renderer.set_font_id(&font_id));
    }

    pub(crate) fn paint_line(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        textarea_properties: &TextAreaProperties,
        line: usize,
        drawing_pos: Pos2,
    ) {
        self.renderers.iter().for_each(|r| {
            r.renderer
                .paint_line(ui, theme, textarea_properties, line, drawing_pos)
        });
    }
}

#[derive(Debug)]
pub(crate) struct RendererEntry {
    layer: u8,
    pub(crate) renderer: Box<dyn Renderer>,
}
