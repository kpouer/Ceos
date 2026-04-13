use crate::ceos::docking::dock_status::{DockStatus, DockType};
use crate::ceos::docking::side_toolbar::SideToolbar;
use crate::event::Event;
use eframe::emath::Vec2;
use egui::{Response, Ui, Widget};
use std::sync::mpsc::Sender;

pub(crate) struct DockManager<'a> {
    sender: &'a Sender<Event>,
    docking_status: &'a mut DockStatus,
}

impl<'a> DockManager<'a> {
    pub(crate) const fn new(sender: &'a Sender<Event>, docking_status: &'a mut DockStatus) -> Self {
        Self {
            sender,
            docking_status,
        }
    }

    fn show_side_panel(&mut self, ui: &mut Ui, title: &str, panel_id: &'static str) -> Response {
        egui::Panel::left(panel_id)
            .resizable(true)
            .size_range(30.0..=ui.available_width())
            .default_size(self.docking_status.side_panel_width)
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.label(title);
            })
            .response
    }
}

impl Widget for DockManager<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
        let toolbar_response = egui::Frame::side_top_panel(ui.style())
            .fill(ui.style().visuals.extreme_bg_color)
            .stroke(egui::Stroke::NONE)
            .show(ui, |ui| {
                ui.set_height(ui.available_height());
                SideToolbar::new(self.sender).ui(ui)
            })
            .inner;

        if let Some(current) = self.docking_status.current() {
            let panel_response = match current {
                DockType::Browser => self.show_side_panel(ui, "Browser", "browser_panel"),
                DockType::Highlight => self.show_side_panel(ui, "Highlight", "highlight_panel"),
            };

            self.docking_status.side_panel_width = panel_response.rect.width();
            panel_response
        } else {
            toolbar_response
        }
    }
}
