use crate::ceos::buffer::Buffer;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::Ceos;
use crate::event::Event::{BufferClosed, BufferLoaded, BufferLoadingStarted, GotoLine};
use eframe::Frame;
use egui::{Align, Context, Direction, Key, Layout, ProgressBar, Ui, Visuals, Widget};
use humansize::{format_size_i, DECIMAL};
use log::{error, info, warn};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::thread;
use textpane::TextPane;
use theme::Theme;

pub(crate) mod frame_history;
pub(crate) mod searchpanel;
pub(crate) mod textpane;
pub mod theme;
pub(crate) mod tools;

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
            self.process_event(ctx, event)
        }

        if let Some(loading_progress) = &self.loading_progress {
            println!("Loading");
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label(format!("Loading {:?}", loading_progress.path));
                    ui.add(
                        ProgressBar::new(
                            loading_progress.current as f32 / loading_progress.size as f32,
                        )
                        .desired_width(300.0),
                    );
                });
            });
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            return;
        }

        self.build_menu_panel(ctx);
        self.build_bottom_panel(ctx);

        egui::CentralPanel::default()
            .frame(egui::containers::Frame::none())
            .show(ctx, |ui| {
                if self.textarea_properties.char_width == 0.0 {
                    let char_width =
                        tools::char_width(self.textarea_properties.font_id.clone(), ui);
                    self.textarea_properties.char_width = char_width;
                }
                self.before_frame();
                TextPane::new(
                    &mut self.textarea_properties,
                    &self.current_command,
                    &self.theme,
                    &self.sender,
                    &self.search_panel.search,
                )
                .ui(ui)
            });
    }
}

impl Ceos {
    fn before_frame(&mut self) {
        if let Some(command) = self.current_command.as_mut() {
            command.before_frame();
        }
        self.textarea_properties.renderer_manager.before_frame();
    }

    fn build_menu_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                self.file_menu(ui);
                self.view_menu(ui);
            });
        });
    }

    fn file_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                self.browse_open_file();
            }
            if ui.button("Save").clicked() {
                self.save_file();
            }
            if ui.button("Close").clicked() {
                self.sender.send(BufferClosed).unwrap();
            }
            if ui.button("Quit").clicked() {
                info!("Quit");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn view_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("View", |ui| {
            if ui.button("☀ Solarized Light").clicked() {
                self.set_theme(Theme::solarized_light(), ui.ctx());
            }
            if ui.button("🌙 Solarized Dark").clicked() {
                self.set_theme(Theme::solarized_dark(), ui.ctx());
            }
            if ui.button("☀ jEdit").clicked() {
                self.set_theme(Theme::jEdit(), ui.ctx());
            }
        });
    }

    fn set_theme(&mut self, theme: Theme, ctx: &Context) {
        let visuals = Visuals::from(&theme);
        self.theme = theme;
        ctx.set_visuals(visuals);
    }

    fn build_bottom_panel(&mut self, ctx: &Context) {
        let mut bottom = egui::TopBottomPanel::bottom("bottom_panel");
        if self.search_panel.search.has_results() {
            bottom = bottom
                .max_height(200.0)
                .default_height(200.0)
                .resizable(true);
        }
        bottom.show(ctx, |ui| {
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
                        if self.try_search() {
                            self.sender
                                .send(GotoLine(Goto::new(self.search_panel.search.line())))
                                .unwrap();
                        } else {
                            self.try_filter_command();
                        }
                    }
                });
                self.status_bar(ui);
            });
            if self.search_panel.search.has_results() {
                self.search_panel.ui(&self.textarea_properties.buffer, ui);
            }
            self.frame_history.ui(ui);
            self.handle_keys(ui);
        });
    }

    fn handle_keys(&mut self, ui: &Ui) {
        #[allow(clippy::collapsible_if)]
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            self.execute_command();
            self.command_buffer.clear();
        } else if ui.input(|i| i.key_pressed(Key::W) && i.modifiers.ctrl) {
            self.sender.send(BufferClosed).unwrap();
        } else if ui.input(|i| i.key_pressed(Key::O) && i.modifiers.ctrl) {
            self.browse_open_file();
        } else if ui.input(|i| i.key_pressed(Key::S) && i.modifiers.ctrl) {
            self.save_file();
        } else if ui.input(|i| i.key_pressed(Key::F3)) {
            if self.search_panel.search.has_results() {
                self.search_panel.search.next();
                self.sender
                    .send(GotoLine(Goto::from(self.search_panel.search.line())))
                    .unwrap();
            }
        } else if ui.input(|i| i.key_pressed(Key::F3) && i.modifiers.shift) {
            if self.search_panel.search.has_results() {
                self.search_panel.search.prev();
                self.sender
                    .send(GotoLine(Goto::from(self.search_panel.search.line())))
                    .unwrap();
            }
        }
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let size = format_size_i(self.textarea_properties.buffer.len(), DECIMAL);
            ui.label(format!("Length: {size}"));
            ui.label(format!(
                "{} lines",
                self.textarea_properties.buffer.line_count()
            ));
        });
    }

    pub(crate) fn browse_open_file(&self) {
        info!("Browse open file");
        if let Some(path) = rfd::FileDialog::new().set_directory("./").pick_file() {
            let path = path.into_os_string();
            let path = path.to_str().unwrap();
            self.open_file(path.into());
        }
    }

    pub(crate) fn open_file(&self, path: PathBuf) {
        info!("Open file {path:?}");
        let sender = self.sender.clone();
        thread::spawn(move || {
            sender.send(BufferClosed).unwrap();
            match Buffer::new_from_file(path, &sender) {
                Ok(buffer) => sender.send(BufferLoaded(buffer)).unwrap(),
                Err(e) => warn!("{:?}", e),
            }
        });
    }

    fn save_file(&mut self) {
        info!("save_file");
        if self.textarea_properties.buffer.dirty {
            if let Some(path) = &self.textarea_properties.buffer.path {
                match File::create(path) {
                    Ok(file) => {
                        let mut file = LineWriter::new(file);
                        self.textarea_properties
                            .buffer
                            .content
                            .iter()
                            .map(|line| &line.content)
                            .for_each(|line| {
                                Self::write(&mut file, line.as_bytes());
                                Self::write(&mut file, b"\n");
                            });
                        self.textarea_properties.buffer.dirty = false;
                    }
                    Err(err) => error!("Unable to save file {path:?} becaues {err}"),
                }
            }
        }
    }

    fn write(file: &mut LineWriter<File>, text: &[u8]) {
        match file.write_all(text) {
            Ok(_) => {}
            Err(err) => error!("{err}"),
        }
    }
}
