use std::sync::mpsc::Sender;
use eframe::epaint::Vec2;
use egui::{Response, Sense, Ui, Widget};
use egui::scroll_area::ScrollBarVisibility::AlwaysHidden;

use crate::ceos::command::Command;
use crate::ceos::gui::textarea::textareaproperties::TextAreaProperties;
use crate::ceos::gui::theme::Theme;
use crate::ceos::gui::widget::gutter::Gutter;
use crate::ceos::gui::widget::textarea::TextArea;
use crate::event::Event;

pub(crate) struct TextPane<'a> {
    textarea_properties: &'a mut TextAreaProperties,
    current_command: &'a Option<Box<dyn Command>>,
    theme: &'a Theme,
    sender: &'a Sender<Event>,
}

impl<'a> TextPane<'a> {
    pub(crate) fn new(
        textarea_properties: &'a mut TextAreaProperties,
        current_command: &'a Option<Box<dyn Command>>,
        theme: &'a Theme,
        sender: &'a Sender<Event>
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            theme,
            sender
        }
    }
}

impl Widget for TextPane<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.max_rect().size();
        let new_scroll_offset = ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let gutter_width = self.textarea_properties.gutter_width();
            println!("gutter size {gutter_width}");
            let current_scroll_offset = &self.textarea_properties.scroll_offset;

            let gutter_size = Vec2::new(gutter_width, ui.available_height());
            let (gutter_rect, response) = ui.allocate_exact_size(gutter_size, Sense::click_and_drag());
            let scroll_result_gutter = egui::ScrollArea::vertical()
                .id_source("gutter")
                .auto_shrink(false)
                .max_width(gutter_width)
                .scroll_bar_visibility(AlwaysHidden)
                .vertical_scroll_offset(current_scroll_offset.y)
                .show_viewport(ui, |ui, rect| {
                    // println!("rect {} gutter_rect {}", rect.size(), gutter_rect.size());
                    Gutter::new(self.textarea_properties, gutter_rect, rect).ui(ui);
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
                        self.sender,
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
        self.textarea_properties.scroll_offset = new_scroll_offset.inner;
        let (_, response) = ui.allocate_exact_size(rect, egui::Sense::click_and_drag());
        response
    }
}
