use std::thread;
use std::time::Duration;

use eframe::Frame;
use egui::Context;
use egui::Event::MouseWheel;
use log::{info, warn};

use crate::ceos::Ceos;
use crate::event::Event::BufferLoaded;
use crate::textarea::buffer::Buffer;

impl eframe::App for Ceos {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        while let Ok(event) = self.receiver.try_recv() {
            self.process_event(event)
        }

        self.handle_input(ctx);
        self.build_menu_panel(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.textarea.show(ui, &self.current_command);
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Command: ");
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.command_buffer),
                    );
                    if response.changed() {
                        self.try_command();
                    }
                });
                ui.label(format!(
                    "Usage : {}, real: {}",
                    self.textarea.buffer().total_length(),
                    self.textarea.buffer().length()
                ));
            });
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_command();
                self.command_buffer.clear();
            }
        });
        ctx.request_repaint_after(Duration::from_millis(1000 / 60));
    }
}

impl Ceos {
    fn build_menu_panel(&self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_file();
                    }
                    if ui.button("Close").clicked() {
                        self.sender
                            .send(BufferLoaded(Default::default()))
                            .unwrap_or_default();
                    }
                    if ui.button("Quit").clicked() {
                        info!("Quit");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
            });
        });
    }

    fn open_file(&self) {
        warn!("Open file");
        let sender = self.sender.clone();
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new().set_directory("./").pick_file() {
                let path = path.into_os_string();
                let path = path.to_str().unwrap();
                match Buffer::try_from(path.to_string()) {
                    Ok(buffer) => sender.send(BufferLoaded(buffer)).unwrap(),
                    Err(e) => warn!("{:?}", e),
                }
            }
        });
    }

    fn handle_input(&mut self, ctx: &Context) {
        ctx.input(|i| {
            i.events.iter().for_each(|event| match event {
                MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => {
                    if delta.y > 0.0 {
                        self.textarea.scroll_up();
                    } else if delta.y < 0.0 {
                        self.textarea.scroll_down();
                    }
                    if delta.x > 0.0 {
                        self.textarea.scroll_left();
                    } else if delta.x < 0.0 {
                        self.textarea.scroll_right();
                    }
                }
                _ => {}
            })
        });
    }
}
