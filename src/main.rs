#![windows_subsystem = "windows"]
use crate::ceos::Ceos;

mod ceos;
mod event;

const INITIAL_WIDTH: f32 = 1024.0;
const INITIAL_HEIGHT: f32 = 768.0;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_inner_size([INITIAL_WIDTH, INITIAL_HEIGHT]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Ceos",
        native_options,
        Box::new(|_cc| Ok(Box::<Ceos>::default())),
    );

    Ok(())
}
