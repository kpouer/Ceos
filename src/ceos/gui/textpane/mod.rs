use crate::ceos::command::Command;
use crate::ceos::command::search::Search;
use crate::ceos::gui::theme::Theme;
use crate::event::Event;
use eframe::epaint::Vec2;
use egui::scroll_area::ScrollBarVisibility::AlwaysHidden;
use egui::{Context, Id, Response, Ui, Widget};
use gutter::Gutter;
use std::sync::mpsc::Sender;
use textarea::TextArea;
use textareaproperties::TextAreaProperties;

pub(crate) mod gutter;
mod position;
pub(crate) mod renderer;
mod selection;
mod textarea;
pub(crate) mod textareaproperties;

#[derive(Debug)]
pub(crate) struct TextPane<'a> {
    textarea_properties: &'a mut TextAreaProperties,
    current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
    search: &'a Search,
    theme: &'a Theme,
    sender: &'a Sender<Event>,
}

impl<'a> TextPane<'a> {
    pub(crate) const fn new(
        textarea_properties: &'a mut TextAreaProperties,
        current_command: &'a Option<Box<dyn Command + Send + Sync + 'static>>,
        theme: &'a Theme,
        sender: &'a Sender<Event>,
        search: &'a Search,
    ) -> Self {
        Self {
            textarea_properties,
            current_command,
            theme,
            sender,
            search,
        }
    }
}

impl Widget for TextPane<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let gutter_width = self.textarea_properties.gutter_width();

            let mut gutter_rect = ui.available_rect_before_wrap();
            gutter_rect.set_width(gutter_width);
            let mut textpane_state = TextPaneState::get(ui.ctx());
            let scroll_result_gutter = egui::ScrollArea::vertical()
                .id_salt("gutter")
                .auto_shrink(false)
                .max_width(gutter_width)
                .scroll_bar_visibility(AlwaysHidden)
                .vertical_scroll_offset(textpane_state.scroll_offset.y)
                .show_viewport(ui, |ui, rect| {
                    Gutter::new(self.textarea_properties, gutter_rect, rect).ui(ui);
                });
            let text_area_rect = ui.available_rect_before_wrap();
            let scroll_result_textarea = egui::ScrollArea::both()
                .id_salt("textarea")
                .auto_shrink(false)
                .scroll_offset(textpane_state.scroll_offset)
                .show_viewport(ui, |ui, rect| {
                    TextArea::new(
                        self.textarea_properties,
                        self.current_command,
                        text_area_rect,
                        rect,
                        self.theme,
                        self.sender,
                        self.search,
                    )
                    .ui(ui)
                });

            let mut offset = scroll_result_textarea.state.offset;
            offset.y = if scroll_result_gutter.state.offset.y != textpane_state.scroll_offset.y {
                scroll_result_gutter.state.offset.y
            } else if scroll_result_textarea.state.offset.y != textpane_state.scroll_offset.y {
                scroll_result_textarea.state.offset.y
            } else {
                textpane_state.scroll_offset.y
            };
            if textpane_state.scroll_offset != offset {
                textpane_state.scroll_offset = offset;
                ui.ctx()
                    .memory_mut(|m| m.data.insert_temp(TextPaneState::id(), textpane_state));
            }
        });
        let (_, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
        response
    }
}

#[derive(Default, Clone)]
pub(crate) struct TextPaneState {
    pub(crate) scroll_offset: Vec2,
}

impl TextPaneState {
    pub(crate) fn id() -> Id {
        Id::new("textpane")
    }

    pub(crate) fn get(ctx: &Context) -> Self {
        ctx.memory(|m| m.data.get_temp::<TextPaneState>(Self::id()))
            .unwrap_or_default()
    }
}
