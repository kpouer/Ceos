use egui::scroll_area::ScrollBarVisibility::AlwaysHidden;
use egui::{Response, Ui, Widget};

use crate::ceos::command::Command;
use crate::ceos::gui::theme::Theme;
use crate::ceos::gui::widget::gutter::Gutter;
use crate::ceos::gui::widget::textarea::TextArea;
use crate::ceos::textarea::textareaproperties::TextAreaProperties;

pub(crate) struct TextPane<'a> {
    textarea_properties: &'a mut TextAreaProperties,
    current_command: &'a Option<Box<dyn Command>>,
    theme: &'a Theme,
}

impl<'a> TextPane<'a> {
    pub(crate) fn new(
        textarea_properties: &'a mut TextAreaProperties,
        current_command: &'a Option<Box<dyn Command>>,
        theme: &'a Theme,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            theme,
        }
    }
}

impl Widget for TextPane<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.max_rect().size();
        let new_scroll_offset = ui.horizontal_top(|ui| {
            let gutter_width = self.textarea_properties.gutter_width();
            let current_scroll_offset = self.textarea_properties.scroll_offset();
            let scroll_result_gutter = egui::ScrollArea::vertical()
                .id_source("gutter")
                .auto_shrink(false)
                .max_width(gutter_width)
                .scroll_bar_visibility(AlwaysHidden)
                .vertical_scroll_offset(current_scroll_offset.y)
                .show_viewport(ui, |ui, rect| {
                    Gutter::new(self.textarea_properties, rect).ui(ui);
                });
            let scroll_result_textarea = egui::ScrollArea::both()
                .id_source("textarea")
                .auto_shrink(false)
                .scroll_offset(*current_scroll_offset)
                .show_viewport(ui, |ui, rect| {
                    TextArea::new(
                        self.textarea_properties,
                        self.current_command,
                        rect,
                        self.theme,
                    )
                    .ui(ui)
                });

            let mut offset = scroll_result_textarea.state.offset;
            offset.y = if scroll_result_gutter.state.offset.y != current_scroll_offset.y {
                scroll_result_gutter.state.offset.y
            } else if scroll_result_textarea.state.offset.y != current_scroll_offset.y {
                scroll_result_textarea.state.offset.y
            } else {
                current_scroll_offset.y
            };
            offset
        });
        self.textarea_properties
            .set_scroll_offset(new_scroll_offset.inner);
        let (_, response) = ui.allocate_exact_size(rect, egui::Sense::click_and_drag());
        response
    }
}
