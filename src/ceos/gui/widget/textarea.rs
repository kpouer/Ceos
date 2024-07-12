use crate::ceos::command::Command;
use crate::textarea::textareaproperties::TextAreaProperties;
use eframe::emath::{Pos2, Rect};
use egui::Widget;

pub(crate) struct TextArea<'a> {
    textarea_properties: &'a TextAreaProperties,
    current_command: &'a Option<Box<dyn Command>>,
    rect: Rect,
}

impl<'a> TextArea<'a> {
    pub(crate) fn new(
        textarea_properties: &'a TextAreaProperties,
        current_command: &'a Option<Box<dyn Command>>,
        rect: Rect,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            rect,
        }
    }
}

impl Widget for TextArea<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_height(self.textarea_properties.text_height());
        let max_rect = ui.max_rect();
        let clip_rect = ui.clip_rect();

        let mut drawing_pos = Pos2::new(max_rect.left(), clip_rect.top());
        let mut virtual_pos = self.rect.left_top();
        let row_range = self.textarea_properties.get_row_range_for_rect(self.rect);
        for line in row_range {
            if let Some(filter_renderer) = &self.current_command {
                filter_renderer.paint_line(
                    ui,
                    &self.textarea_properties,
                    line,
                    virtual_pos,
                    drawing_pos,
                );
            }

            self.textarea_properties.renderers().iter().for_each(|r| {
                r.paint_line(
                    ui,
                    &self.textarea_properties,
                    line,
                    virtual_pos,
                    drawing_pos,
                )
            });

            // self.gutter.paint_line(ui, self, line, Pos2::new(max_rect.left_top().x, pos.y));
            drawing_pos.y += self.textarea_properties.line_height();
            virtual_pos.y += self.textarea_properties.line_height();
        }

        let text_bounds = self.textarea_properties.text_bounds();
        let (_, response) = ui.allocate_exact_size(text_bounds, egui::Sense::click_and_drag());
        response
    }
}
