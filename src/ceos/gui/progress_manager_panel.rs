use crate::ceos::progress_manager::ProgressManager;
use eframe::emath::Align;
use egui::{Layout, ProgressBar, Response, Ui, Widget};

pub(crate) struct ProgressManagerPanel<'a> {
    progress_manager: &'a ProgressManager,
}

impl<'a> ProgressManagerPanel<'a> {
    pub const fn new(progress_manager: &'a ProgressManager) -> Self {
        Self { progress_manager }
    }
}

impl Widget for ProgressManagerPanel<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    self.progress_manager
                        .iter()
                        .map(|(_key, progress)| {
                            let percent = progress.percent();
                            ProgressBar::new(percent)
                                .text(format!(
                                    "{} {}/100 %",
                                    progress.label,
                                    (percent * 100.0) as usize
                                ))
                                .corner_radius(10.0)
                                .desired_width(600.0)
                        })
                        .for_each(|progress_bar| {
                            ui.add(progress_bar);
                        });
                })
            })
            .response
    }
}
