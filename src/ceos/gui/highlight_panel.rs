use crate::ceos::highlight::manager::HighlightManager;
use crate::event::Event;
use egui::{Color32, Ui};
use std::sync::mpsc::Sender;

pub(crate) struct HighlightPanel<'a> {
    pub(crate) manager: &'a mut HighlightManager,
    pub(crate) sender: &'a Sender<Event>,
}

impl<'a> HighlightPanel<'a> {
    pub(crate) fn new(manager: &'a mut HighlightManager, sender: &'a Sender<Event>) -> Self {
        Self { manager, sender }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Highlights");
            ui.separator();

            let mut to_remove = None;
            for (i, highlight) in self.manager.iter().enumerate() {
                ui.horizontal(|ui| {
                    let (r, g, b, _) = highlight.color.to_tuple();
                    ui.label(
                        egui::RichText::new(&highlight.text)
                            .background_color(highlight.color)
                            .color(if (r as u32 + g as u32 + b as u32) / 3 > 128 {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            }),
                    );
                    if highlight.case_insensitive {
                        ui.label("(i)");
                    }
                    if ui.button("x").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(i) = to_remove {
                let _ = self.sender.send(Event::RemoveHighlight(i));
            }

            ui.separator();
            ui.label("Add highlight (command line):");
            ui.label("highlight text [case_insensitive]");
        });
    }
}
