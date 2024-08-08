use crate::ceos::gui::textpane::renderer::Renderer;
use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::Pos2;
use eframe::epaint::FontId;
use egui::Ui;

pub(crate) const BASE_LAYER: u8 = 0;
pub(crate) const HIGHLIGHT_LAYER: u8 = 50;
pub(crate) const TEXT_LAYER: u8 = 100;

#[derive(Default)]
pub(crate) struct RendererManager {
    pub(crate) renderers: Vec<RendererEntry>,
}

impl RendererManager {
    pub(crate) fn add_renderer(&mut self, layer: u8, renderer: Box<dyn Renderer>) {
        let entry = RendererEntry { layer, renderer };
        for (i, e) in self.renderers.iter().enumerate() {
            if entry.layer < e.layer {
                self.renderers.insert(i, entry);
                return;
            }
        }

        self.renderers.push(entry);
    }

    pub(crate) fn set_font_id(&mut self, font_id: FontId) {
        self.renderers
            .iter_mut()
            .for_each(|r| r.renderer.set_font_id(font_id.clone()));
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

pub(crate) struct RendererEntry {
    layer: u8,
    pub(crate) renderer: Box<dyn Renderer>,
}
