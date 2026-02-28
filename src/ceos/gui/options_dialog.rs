use egui::{Context, Ui};
use crate::ceos::options::Options;
use log::warn;

#[derive(Debug)]
pub(crate) struct OptionsDialog;

impl OptionsDialog {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn ui(&mut self, ctx: &Context, options: &mut Options, open: &mut bool) {
        egui::Window::new("Options")
            .open(open)
            .resizable(true)
            .show(ctx, |ui| {
                self.content_ui(ui, options);
            });
    }

    fn content_ui(&self, ui: &mut Ui, options: &mut Options) {
        ui.vertical(|ui| {
            ui.heading("Settings");
            let response = ui.checkbox(&mut options.compression, "Compression");
            if response.changed() {
                if let Err(e) = options.save() {
                    warn!("Unable to save ceos.toml: {e}");
                }
            }
        });
    }
}
