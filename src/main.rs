#![windows_subsystem = "windows"]
use crate::ceos::Ceos;

mod ceos;
mod event;

const INITIAL_WIDTH: usize = 1024;
const INITIAL_HEIGHT: usize = 768;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_inner_size([INITIAL_WIDTH as f32, INITIAL_HEIGHT as f32])
            .with_min_inner_size([INITIAL_WIDTH as f32, INITIAL_HEIGHT as f32]),
        ..Default::default()
    };
    // thread::spawn(move|| run_loop(image_data_sender, interaction_receiver));
    let _ = eframe::run_native(
        "Ceos",
        native_options,
        Box::new(|_cc| Ok(Box::<Ceos>::default())),
    );

    Ok(())
}
