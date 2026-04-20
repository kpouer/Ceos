use crate::ceos::progress_manager::ProgressManager;
use eframe::emath::Align;
use egui::{Layout, ProgressBar, Response, Ui, Widget};
use num_traits::{ToPrimitive, Zero};
use std::fmt::Debug;
use std::ops::AddAssign;

pub(crate) struct ProgressManagerPanel<'a, T>
where
    T: PartialOrd + ToPrimitive + Zero + Default + AddAssign + Debug,
{
    progress_manager: &'a ProgressManager<T>,
}

impl<'a, T> ProgressManagerPanel<'a, T>
where
    T: PartialOrd + ToPrimitive + Zero + Default + AddAssign + Debug,
{
    pub const fn new(progress_manager: &'a ProgressManager<T>) -> Self {
        Self { progress_manager }
    }
}

impl<T> Widget for ProgressManagerPanel<'_, T>
where
    T: PartialOrd + ToPrimitive + Zero + Default + AddAssign + Debug,
{
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
