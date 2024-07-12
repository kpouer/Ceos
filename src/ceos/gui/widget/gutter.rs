use eframe::emath::{Rect, Vec2};
use eframe::epaint::{Color32, Stroke};
use egui::{Response, Ui, Widget};

use crate::textarea::buffer::Buffer;
use crate::textarea::textareaproperties::TextAreaProperties;

pub(crate) struct Gutter<'a> {
    textarea_properties: &'a TextAreaProperties,
    rect: Rect,
}

impl<'a> Gutter<'a> {
    pub(crate) fn new(textarea: &'a TextAreaProperties, rect: Rect) -> Self {
        Self {
            textarea_properties: textarea,
            rect,
        }
    }
}

impl Widget for Gutter<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut max_rect = ui.max_rect();
        let painter = ui.painter();
        let gutter_width = self.textarea_properties.gutter_width();
        max_rect.set_width(gutter_width);
        let mut gutter_rect = ui.clip_rect();
        gutter_rect.set_width(gutter_width);
        painter.rect(gutter_rect, 0.0, Color32::LIGHT_GRAY, Stroke::NONE);
        let mut pos = gutter_rect.right_top();
        let initial_x = pos.x;
        pos.x -= self.textarea_properties.char_width();
        let painter = ui.painter();
        let row_range = self.textarea_properties.get_row_range_for_rect(self.rect);
        for line in row_range {
            let line_number = line + 1;
            let text_width =
                (1 + line_number.ilog10()) as f32 * self.textarea_properties.char_width();
            // pos.x = initial_x + gutter_width - self.textarea_properties.char_width() - text_width;
            if line % 10 == 0 {
                println!("{line_number} text_width {text_width} posx {}", pos.x);
            }
            painter.text(
                pos,
                egui::Align2::RIGHT_TOP,
                format!("{}", line_number),
                self.textarea_properties.font_id().clone(),
                ui.visuals().text_color(),
            );
            pos.y += self.textarea_properties.line_height();
        }

        let size = Vec2::new(gutter_width, self.textarea_properties.text_height());
        let (_, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        response
    }
}

pub(crate) fn gutter_width(char_width: f32, buffer: &Buffer) -> f32 {
    if buffer.line_count() == 0 {
        char_width * 2.0
    } else {
        char_width * (2.0 + 1.0 + buffer.line_count().ilog10() as f32)
    }
}
