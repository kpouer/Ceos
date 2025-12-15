use crate::ceos::command::Command;
use crate::ceos::command::direct::goto::Goto;
use crate::ceos::command::search::Search;
use crate::ceos::gui::frame_history::FrameHistory;
use crate::ceos::gui::searchpanel::SearchPanel;
use crate::ceos::gui::textpane::TextPane;
use crate::ceos::options::Options;
use crate::ceos::progress_manager::{BUFFER_LOADING, BUFFER_SAVING, ProgressManager};
use crate::event::Event;
use crate::event::Event::{BufferClosed, BufferLoaded, GotoLine};
use Event::NewFont;
use buffer::buffer::Buffer;
use eframe::Frame;
use eframe::emath::Align;
use egui::{Context, Key, Layout, ProgressBar, Ui, Visuals, Widget};
use gui::textpane::textareaproperties::TextAreaProperties;
use gui::theme::Theme;
use humansize::{DECIMAL, format_size_i};
use log::{debug, error, info, warn};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

pub(crate) mod buffer;
pub(crate) mod command;
pub(crate) mod gui;
mod options;
mod progress_manager;
mod syntax;
mod tools;

#[derive(Debug)]
pub(crate) struct Ceos {
    textarea_properties: TextAreaProperties,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    command_buffer: String,
    current_command: Option<Box<dyn Command + Send + Sync + 'static>>,
    frame_history: FrameHistory,
    search_panel: SearchPanel,
    theme: Theme,
    initialized: bool,
    progress_manager: ProgressManager,
    show_options: bool,
    options: Options,
}

impl Default for Ceos {
    fn default() -> Self {
        let (user_input_sender, user_input_receiver) = channel::<Event>();
        let search_panel = SearchPanel::new(user_input_sender.clone());
        Self {
            sender: user_input_sender.clone(),
            receiver: user_input_receiver,
            textarea_properties: TextAreaProperties::new(user_input_sender),
            command_buffer: String::new(),
            current_command: None,
            frame_history: Default::default(),
            search_panel,
            theme: Theme::default(),
            initialized: false,
            progress_manager: Default::default(),
            show_options: false,
            options: Options::load(),
        }
    }
}

impl Ceos {
    pub(crate) fn process_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::ClearCommand => {
                self.command_buffer = String::new();
                self.current_command = None;
            }
            Event::SetCommand(command) => {
                self.command_buffer = command;
                self.try_filter_command();
            }
            Event::OpenFile(path) => self.open_file(path),
            Event::BufferLoadingStarted(path, size) => {
                self.progress_manager
                    .add(BUFFER_LOADING.into(), format!("Loading {path:?}"), size)
            }
            Event::BufferLoading(_, current, _) => {
                self.progress_manager.update(BUFFER_LOADING, current)
            }
            Event::BufferSavingStarted(path, size) => {
                self.progress_manager
                    .add(BUFFER_SAVING.into(), format!("Saving {path:?}"), size)
            }
            Event::BufferSaving(_, current, _) => {
                self.progress_manager.update(BUFFER_SAVING, current)
            }
            Event::BufferSaved(path) => {
                self.progress_manager.remove(BUFFER_SAVING);
                // Marquer le buffer comme non-dirty si c'est le même fichier
                if let Some(current_path) = &self.textarea_properties.buffer.path {
                    if current_path == &path {
                        self.textarea_properties.buffer.dirty = false;
                    }
                }
            }
            Event::BufferSaveFailed(_) => {
                // Retirer la progression en cas d'échec
                self.progress_manager.remove(BUFFER_SAVING);
            }
            BufferLoaded(buffer) => {
                self.progress_manager.remove(BUFFER_LOADING);
                self.textarea_properties.set_buffer(buffer);
            }
            BufferClosed => self.textarea_properties.set_buffer(Buffer::new(self.sender.clone())),
            GotoLine(goto) => goto.execute(ctx, &mut self.textarea_properties),
            NewFont(font_id) => self.textarea_properties.set_font_id(font_id),
            Event::OperationStarted(label, length) => self.progress_manager.add(label.clone(), label, length),
            Event::OperationProgress(label, value) => self.progress_manager.update(&label, value),
            Event::OperationIncrement(label, amount) => self.progress_manager.increment(&label, amount),
            Event::OperationFinished(label) => self.progress_manager.remove(&label),
        }
    }
}

impl Ceos {
    pub(crate) fn try_search(&mut self) -> bool {
        if let Ok(mut search) = Search::try_from(self.command_buffer.as_str()) {
            search.init(&self.textarea_properties.buffer);
            self.search_panel.search = search;
            return true;
        }
        false
    }

    pub(crate) fn try_filter_command(&mut self) {
        let command_str = self.command_buffer.as_str();
        if let Ok(command) =
            crate::ceos::command::filter::linefilter::LineFilter::try_from(command_str)
        {
            self.current_command = Some(Box::new(command));
        } else if let Ok(command) =
            crate::ceos::command::filter::columnfilter::ColumnFilter::try_from(command_str)
        {
            self.current_command = Some(Box::new(command));
        } else if let Ok(command) =
            crate::ceos::command::filter::linedrop::LineDrop::try_from(command_str)
        {
            self.current_command = Some(Box::new(command));
        } else {
            self.current_command = None;
        }

        if let Some(command) = &self.current_command {
            debug!("Found command {}", command);
        }
    }

    pub(crate) fn execute_command(&mut self) {
        if let Some(command) = self.current_command.take() {
            info!("Execute command {}", command);
            let mut tmp_buffer = Buffer::new(self.sender.clone());
            std::mem::swap(&mut tmp_buffer, &mut self.textarea_properties.buffer);
            let sender = self.sender.clone();
            std::thread::spawn(move || {
                command.execute(&mut tmp_buffer);
                sender.send(Event::BufferLoaded(tmp_buffer)).unwrap();
            });

        } else if let Ok(command) = Event::try_from(self.command_buffer.as_str()) {
            self.sender.send(command).unwrap();
        }
    }
}

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

        if !self.progress_manager.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
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
                });
            });
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            return;
        }

        self.build_menu_panel(ctx);
        self.build_options_window(ctx);
        self.build_bottom_panel(ctx);

        egui::CentralPanel::default()
            .frame(egui::containers::Frame::NONE)
            .show(ctx, |ui| {
                if self.textarea_properties.char_width == 0.0 {
                    let char_width =
                        gui::tools::char_width(self.textarea_properties.font_id.clone(), ui);
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

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                self.file_menu(ui);
                self.view_menu(ui);
                self.options_menu(ui);
                self.debug_menu(ui);
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

    fn options_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Options", |ui| {
            if ui.button("Options…").clicked() {
                self.show_options = true;
            }
            // Quick toggle directly in the menu as well (optional convenience)
            ui.separator();
            let response = ui.checkbox(&mut self.options.compression, "Compression");
            if response.changed() && let Err(e) = self.options.save() {
                warn!("Impossible d'enregistrer ceos.toml: {e}");
            }
        });
    }

    fn debug_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Debug", |ui| {
            let (
                line_count,
                group_count,
                compressed,
                decompressed,
                decomressed_line_count,
                compressed_size,
            ) = {
                let buffer = &self.textarea_properties.buffer;
                (
                    buffer.line_count(),
                    buffer.group_count(),
                    buffer.compressed_group_count(),
                    buffer.decompressed_group_count(),
                    buffer.decompressed_line_count(),
                    buffer.compressed_size(),
                )
            };

            ui.label(format!("Lignes du buffer: {}", line_count));
            ui.label(format!("Nombre de groupes: {}", group_count));
            ui.label(format!("Groupes compressés: {}", compressed));
            ui.label(format!("Groupes décompressés: {}", decompressed));
            ui.label(format!("Lignes décompressées: {}", decomressed_line_count));
            ui.label(format!(
                "Compressed size: {}",
                format_size_i(compressed_size, DECIMAL)
            ));

            ui.separator();
            if ui.button("Compress").clicked() {
                self.textarea_properties.buffer.compress_all_groups();
            }
        });
    }

    fn build_options_window(&mut self, ctx: &Context) {
        let mut open = self.show_options;
        egui::Window::new("Options")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Paramètres");
                    let response = ui.checkbox(&mut self.options.compression, "Compression");
                    if response.changed() && let Err(e) = self.options.save() {
                        warn!("Impossible d'enregistrer ceos.toml: {e}");
                    }
                });
            });
        self.show_options = open;
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
            let _ = self.sender.send(BufferClosed);
        } else if ui.input(|i| i.key_pressed(Key::O) && i.modifiers.ctrl) {
            self.browse_open_file();
        } else if ui.input(|i| i.key_pressed(Key::S) && i.modifiers.ctrl) {
            self.save_file();
        } else if ui.input(|i| i.key_pressed(Key::F3)) {
            if self.search_panel.search.has_results() {
                self.search_panel.search.next();
                let _ = self
                    .sender
                    .send(GotoLine(Goto::from(self.search_panel.search.line())));
            }
        } else if ui.input(|i| i.key_pressed(Key::F3) && i.modifiers.shift) {
            if self.search_panel.search.has_results() {
                self.search_panel.search.prev();
                let _ = self
                    .sender
                    .send(GotoLine(Goto::from(self.search_panel.search.line())));
            }
        }
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let size = format_size_i(self.textarea_properties.buffer.len(), DECIMAL);
            ui.label(format!("Length: {size}"));
            let mem = format_size_i(self.textarea_properties.buffer.mem(), DECIMAL);
            ui.label(format!("Mem: {mem}"));
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
            match Buffer::new_from_file(path, sender.clone()) {
                Ok(buffer) => sender.send(BufferLoaded(buffer)).unwrap(),
                Err(e) => warn!("{:?}", e),
            }
        });
    }

    fn save_file(&mut self) {
        info!("save_file");
        if self.textarea_properties.buffer.dirty
            && let Some(path) = &self.textarea_properties.buffer.path
        {
            let path = path.clone();
            // Préparer un snapshot des lignes à écrire pour l'écriture en thread
            let mut total_size: usize = 0;
            let mut lines: Vec<Vec<u8>> = Vec::new();
            for group in self.textarea_properties.buffer.line_groups() {
                let cow = group.lines();
                for line in cow.as_ref().iter() {
                    let bytes: Vec<u8> = line.content().as_bytes().to_vec();
                    total_size += bytes.len() + 1; // +1 pour le saut de ligne
                    lines.push(bytes);
                }
            }

            let sender = self.sender.clone();
            thread::spawn(move || {
                // Démarrer la progression
                let _ = sender.send(Event::BufferSavingStarted(path.clone(), total_size));
                match File::create(&path) {
                    Ok(file) => {
                        let mut file = LineWriter::new(file);
                        let mut current: usize = 0;
                        for bytes in lines.iter() {
                            if let Err(err) = file.write_all(bytes) {
                                error!("{err}");
                                let _ = sender.send(Event::BufferSaveFailed(path.clone()));
                                return;
                            }
                            if let Err(err) = file.write_all(b"\n") {
                                error!("{err}");
                                let _ = sender.send(Event::BufferSaveFailed(path.clone()));
                                return;
                            }
                            current += bytes.len() + 1;
                            let _ = sender.send(Event::BufferSaving(path.clone(), current, total_size));
                        }
                        // Fin de progression
                        let _ = sender.send(Event::BufferSaved(path.clone()));
                    }
                    Err(err) => {
                        error!("Unable to save file {path:?} becaues {err}");
                        let _ = sender.send(Event::BufferSaveFailed(path.clone()));
                    }
                }
            });
        }
    }

    fn write(file: &mut LineWriter<File>, text: &[u8]) {
        match file.write_all(text) {
            Ok(_) => {}
            Err(err) => error!("{err}"),
        }
    }
}
