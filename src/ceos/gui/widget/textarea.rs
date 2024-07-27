use crate::ceos::command::Command;
use crate::ceos::gui::textarea::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use eframe::emath::{Pos2, Rect};
use egui::Widget;

pub(crate) struct TextArea<'a> {
    textarea_properties: &'a TextAreaProperties,
    current_command: &'a Option<Box<dyn Command>>,
    rect: Rect,
    theme: &'a Theme,
}

impl<'a> TextArea<'a> {
    pub(crate) fn new(
        textarea_properties: &'a TextAreaProperties,
        current_command: &'a Option<Box<dyn Command>>,
        rect: Rect,
        theme: &'a Theme,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            rect,
            theme,
        }
    }
}

impl Widget for TextArea<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_height(self.textarea_properties.text_height());
        let mut drawing_pos = Pos2::new(ui.max_rect().left(), ui.clip_rect().top());
        let mut virtual_pos = self.rect.left_top();
        let row_range = self.textarea_properties.get_row_range_for_rect(self.rect);
        row_range.into_iter().for_each(|line| {
            if let Some(filter_renderer) = &self.current_command {
                filter_renderer.paint_line(
                    ui,
                    self.theme,
                    self.textarea_properties,
                    line,
                    virtual_pos,
                    drawing_pos,
                );
            }

            self.textarea_properties.renderers().iter().for_each(|r| {
                r.paint_line(
                    ui,
                    self.theme,
                    self.textarea_properties,
                    line,
                    virtual_pos,
                    drawing_pos,
                )
            });

            drawing_pos.y += self.textarea_properties.line_height();
            virtual_pos.y += self.textarea_properties.line_height();
        });

        let text_bounds = self.textarea_properties.text_bounds();
        let (_, response) = ui.allocate_exact_size(text_bounds, egui::Sense::click_and_drag());
        response
    }
}
