use eframe::emath::{Rect, Vec2};
use eframe::epaint::{Stroke, StrokeKind};
use egui::{Response, Ui, Widget};

use crate::ceos::gui::textpane::textareaproperties::TextAreaProperties;

pub(crate) struct Gutter<'a> {
    textarea_properties: &'a TextAreaProperties,
    drawing_rect: Rect,
    virtual_rect: Rect,
}

impl<'a> Gutter<'a> {
    pub(crate) const fn new(
        textarea: &'a TextAreaProperties,
        drawing_rect: Rect,
        virtual_rect: Rect,
    ) -> Self {
        Self {
            textarea_properties: textarea,
            drawing_rect,
            virtual_rect,
        }
    }
}

impl Widget for Gutter<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let gutter_width = self.textarea_properties.gutter_width();
        let painter = ui.painter();
        painter.rect(
            self.drawing_rect,
            0.0,
            ui.visuals().faint_bg_color,
            Stroke::NONE,
            StrokeKind::Inside,
        );
        let mut pos = self.drawing_rect.right_top();
        pos.x -= self.textarea_properties.char_width;
        let row_range = self
            .textarea_properties
            .get_row_range_for_rect(self.virtual_rect);
        row_range.into_iter().for_each(|line| {
            painter.text(
                pos,
                egui::Align2::RIGHT_TOP,
                format!("{}", line + 1),
                self.textarea_properties.font_id.clone(),
                ui.visuals().text_color(),
            );
            pos.y += self.textarea_properties.line_height;
        });

        let mut size = Vec2::new(gutter_width, self.textarea_properties.text_height());
        if ui.available_height() > size.y {
            size.y = ui.available_height();
        }
        let (_, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        response
    }
}

pub(crate) const fn gutter_width(char_width: f32, line_count: usize) -> f32 {
    if line_count == 0 {
        char_width * 3.0
    } else {
        char_width * (2.0 + 1.0 + line_count.ilog10() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(10.0, 0, 30.0)]
    #[case(10.0, 1, 30.0)]
    #[case(10.0, 9, 30.0)]
    #[case(10.0, 10, 40.0)]
    #[case(10.0, 99, 40.0)]
    #[case(10.0, 100, 50.0)]
    fn test_gutter_width(
        #[case] char_width: f32,
        #[case] line_count: usize,
        #[case] expected: f32,
    ) {
        assert_eq!(gutter_width(char_width, line_count), expected);
    }
}
