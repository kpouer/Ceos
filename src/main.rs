mod ceos;
mod event;
mod textarea;

use crate::ceos::Ceos;

const INITIAL_WIDTH: usize = 1024;
const INITIAL_HEIGHT: usize = 768;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([INITIAL_WIDTH as f32, INITIAL_HEIGHT as f32])
            .with_min_inner_size([INITIAL_WIDTH as f32, INITIAL_HEIGHT as f32]),
        ..Default::default()
    };
    // thread::spawn(move|| run_loop(image_data_sender, interaction_receiver));
    eframe::run_native(
        "Ceos",
        native_options,
        Box::new(|_cc| Ok(Box::<Ceos>::default())),
    );

    Ok(())
}

// fn run_loop(image_data_sender: Sender<ImageData>, interaction_receiver: Receiver<Interaction>) {
//     static DEFAULT_SLEEP: Duration = Duration::from_millis(100);
//     let params = Params::from(DEFAULT_FRACTAL);
//     let mut fractal_renderer = FractalRenderer::new(INITIAL_WIDTH, INITIAL_HEIGHT, params);
//     loop {
//         let start = std::time::Instant::now();
//         let image = fractal_renderer.compute_and_build_image();
//         image_data_sender.send(image).unwrap();
//         let elapsed = start.elapsed();
//         if elapsed > DEFAULT_SLEEP {
//             sleep(DEFAULT_SLEEP - elapsed);
//         }
//     }
// }
