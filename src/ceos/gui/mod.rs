use eframe::epaint::FontId;
use eframe::Frame;
use egui::Event::{MouseWheel, Zoom};
use egui::{Context, Visuals, Widget};
use log::{error, info, warn};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::thread;

use crate::ceos::gui::widget::textpane::TextPane;
use crate::ceos::textarea::buffer::Buffer;
use crate::ceos::Ceos;
use crate::event::Event::{BufferClosed, BufferLoaded, NewFont};
use theme::Theme;

pub(crate) mod frame_history;
pub mod theme;
pub(crate) mod tools;
pub(crate) mod widget;

impl eframe::App for Ceos {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if !self.initialized {
            let theme = Theme::default();
            self.set_theme(theme, ctx);
            self.initialized = true;
        }
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        while let Ok(event) = self.receiver.try_recv() {
            self.process_event(event)
        }

        self.handle_input(ctx);
        self.build_menu_panel(ctx);
        self.build_bottom_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.textarea.char_width() == 0.0 {
                let char_width = tools::char_width(self.textarea.font_id().clone(), ui);
                self.textarea.set_char_width(char_width);
            }
            TextPane::new(&mut self.textarea, &self.current_command, &self.theme).ui(ui)
        });
    }
}

impl Ceos {
    fn build_menu_panel(&mut self, ctx: &Context) {
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
                        self.sender.send(BufferClosed).unwrap();
                    }
                    if ui.button("Quit").clicked() {
                        info!("Quit");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("☀ Solarized Light").clicked() {
                        self.set_theme(Theme::solarized_light(), ctx);
                    }
                    if ui.button("🌙 Solarized Dark").clicked() {
                        self.set_theme(Theme::solarized_dark(), ctx);
                    }
                    if ui.button("☀ jEdit").clicked() {
                        self.set_theme(Theme::jEdit(), ctx);
                    }
                });
            });
        });
    }

    fn set_theme(&mut self, theme: Theme, ctx: &Context) {
        let visuals = Visuals::from(&theme);
        self.theme = theme;
        ctx.set_visuals(visuals);
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
                sender.send(BufferClosed).unwrap();
                match Buffer::new_from_file(path.to_string()) {
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
                    Self::write(&mut file, line.as_bytes());
                    Self::write(&mut file, b"\n");
                })
        }
    }

    fn write(file: &mut LineWriter<File>, text: &[u8]) {
        match file.write_all(text) {
            Ok(_) => {}
            Err(err) => error!("{err}"),
        }
    }

    fn handle_input(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if let Some(file) = i.raw.dropped_files.first() {
                if let Some(path) = &file.path {
                    let path = path.to_string_lossy();
                    let path = path.to_string();
                    let sender = self.sender.clone();
                    thread::spawn(move || {
                        sender.send(BufferClosed).unwrap();
                        match Buffer::new_from_file(path) {
                            Ok(buffer) => sender.send(BufferLoaded(buffer)).unwrap(),
                            Err(e) => warn!("{:?}", e),
                        }
                    });
                }
            }
            i.events.iter().for_each(|event| match event {
                MouseWheel {
                    unit: _,
                    delta,
                    modifiers: _,
                } => {
                    #[cfg(target_os = "macos")]
                    if i.modifiers.command {
                        self.zoom(delta.y);
                    }
                    #[cfg(not(target_os = "macos"))]
                    if i.modifiers.ctrl {
                        self.zoom(delta.y);
                    }
                }
                Zoom(delta) => {
                    if *delta < 1.0 {
                        self.zoom(-(*delta));
                    } else if *delta > 1.0 {
                        self.zoom(*delta);
                    }
                }
                _ => {}
            })
        });
    }

    fn zoom(&self, delta: f32) {
        let current_font_size = self.textarea.font_id().size;
        let new_font_size = current_font_size + delta;
        if new_font_size < 1.0 {
            return;
        }

        self.sender
            .send(NewFont(FontId::new(
                new_font_size,
                egui::FontFamily::Monospace,
            )))
            .unwrap()
    }
}
