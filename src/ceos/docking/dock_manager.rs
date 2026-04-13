use crate::ceos::docking::dock_status::{DockStatus, DockType};
use crate::ceos::docking::side_toolbar::SideToolbar;
use crate::event::Event;
use egui::{Response, Ui, Widget};
use std::sync::mpsc::Sender;

pub(crate) struct DockManager<'a> {
    sender: &'a Sender<Event>,
    docking_status: &'a mut DockStatus,
}

impl<'a> DockManager<'a> {
    pub(crate) fn new(sender: &'a Sender<Event>, docking_status: &'a mut DockStatus) -> Self {
        Self {
            sender,
            docking_status,
        }
    }
}

impl Widget for DockManager<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = SideToolbar::new(self.sender).ui(ui);
        if let Some(current) = self.docking_status.current() {
            match current {
                DockType::Browser => {
                    egui::Panel::left("browser_panel")
                        .resizable(true)
                        .size_range(50.0..=ui.available_width())
                        .default_size(self.docking_status.side_panel_width)
                        .show_inside(ui, |ui| {
                            self.docking_status.side_panel_width = ui.available_width();
                            ui.label("Browser");
                        })
                        .response
                }
                DockType::Highlight => {
                    egui::Panel::left("highlight_panel")
                        .resizable(true)
                        .size_range(50.0..=ui.available_width())
                        .default_size(self.docking_status.side_panel_width)
                        .show_inside(ui, |ui| {
                            self.docking_status.side_panel_width = ui.available_width();
                            ui.label("Highlight");
                        })
                        .response
                }
            }
        } else {
            response
        }
    }
}
