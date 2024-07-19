use std::fs::File;
use std::io::{LineWriter, Write};
use std::thread;

use eframe::Frame;
use egui::{Context, Widget};
use log::{info, warn};

use crate::ceos::gui::widget::textpane::TextPane;
use crate::ceos::Ceos;
use crate::event::Event::BufferLoaded;
use crate::textarea::buffer::Buffer;

pub(crate) mod frame_history;
pub(crate) mod tools;
pub(crate) mod widget;

impl eframe::App for Ceos {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        while let Ok(event) = self.receiver.try_recv() {
            self.process_event(event)
        }

        self.handle_input(ctx);
        self.build_menu_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.textarea.char_width() == 0.0 {
                let char_width = tools::char_width(self.textarea.font_id().clone(), ui);
                self.textarea.set_char_width(char_width);
            }
            TextPane::new(&mut self.textarea, &self.current_command).ui(ui)
        });
        self.build_bottom_panel(ctx);
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
                    if ui.button("Save").clicked() {
                        self.save_file();
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

    fn build_bottom_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Command: ");
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.command_buffer),
                    );
                    ui.memory_mut(|memory| {
                        memory.request_focus(response.id);
                    });
                    if response.changed() {
                        self.try_filter_command();
                    }
                });
                ui.label(format!("Length: {}", self.textarea.buffer().len(),));
            });
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_command();
                self.command_buffer.clear();
            }
            self.frame_history.ui(ui);
        });
    }

    fn open_file(&self) {
        info!("Open file");
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

    fn save_file(&self) {
        info!("save_file");
        let path = self.textarea.buffer().path();
        if !path.is_empty() {
            let file = File::create(path).unwrap();
            let mut file = LineWriter::new(file);
            self.textarea
                .buffer()
                .content()
                .iter()
                .map(|line| line.content())
                .for_each(|line| {
                    file.write_all(line.as_bytes());
                    file.write_all(b"\n");
                })
        }
    }

    fn handle_input(&mut self, _: &Context) {
        // ctx.input(|i| {
        //     i.events.iter().for_each(|event| match event {
        //         MouseWheel {
        //             unit: _,
        //             delta,
        //             modifiers: _,
        //         } => {
        //             if delta.y > 0.0 {
        //                 self.textarea.scroll_up();
        //             } else if delta.y < 0.0 {
        //                 self.textarea.scroll_down();
        //             }
        //             if delta.x > 0.0 {
        //                 self.textarea.scroll_left();
        //             } else if delta.x < 0.0 {
        //                 self.textarea.scroll_right();
        //             }
        //         }
        //         _ => {}
        //     })
        // });
    }
}
