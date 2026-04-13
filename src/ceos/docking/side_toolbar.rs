use crate::event::Event;
use egui::{Align, Layout, Response, Ui, Vec2, Widget};
use std::sync::mpsc::Sender;

pub(crate) struct SideToolbar<'a> {
    sender: &'a Sender<Event>,
}

impl<'a> SideToolbar<'a> {
    pub(crate) const fn new(sender: &'a Sender<Event>) -> Self {
        Self { sender }
    }
}

impl Widget for SideToolbar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.set_width(20.0);
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 5.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                if ui.button("📁").on_hover_text("Open file browser").clicked() {
                    let _ = self.sender.send(Event::ShowBrowser);
                }
                if ui.button("🪄").on_hover_text("Highlight").clicked() {
                    let _ = self.sender.send(Event::ShowHighlight);
                }
            });
        })
        .response
    }
}
