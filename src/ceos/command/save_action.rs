use crate::ceos::buffer::buffer::Buffer;
use crate::ceos::command::Action;
use crate::event::Event;
use flate2::Compression;
use flate2::write::GzEncoder;
use log::error;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(crate) struct SaveAction {
    sender: Sender<Event>,
    path: PathBuf,
}

impl SaveAction {
    pub(crate) fn new<T: Into<PathBuf>>(sender: Sender<Event>, path: T) -> SaveAction {
        Self {
            sender,
            path: path.into(),
        }
    }

    fn write_lines(&self, buffer: &Buffer, writer: &mut impl Write) -> bool {
        let total_size = buffer.len();
        let mut current = 0;
        for group in buffer.line_groups() {
            let cow = group.lines();
            for line in cow.as_ref().iter() {
                let bytes: Vec<u8> = line.content().as_bytes().to_vec();
                if let Err(err) = writer.write_all(&bytes) {
                    error!("{err}");
                    let _ = self
                        .sender
                        .send(Event::BufferSaveFailed(self.path.to_path_buf()));
                    return true;
                }
                if let Err(err) = writer.write_all(b"\n") {
                    error!("{err}");
                    let _ = self
                        .sender
                        .send(Event::BufferSaveFailed(self.path.to_path_buf()));
                    return true;
                }
                current += bytes.len() + 1;
                let _ = self.sender.send(Event::BufferSaving(
                    self.path.to_path_buf(),
                    current,
                    total_size,
                ));
            }
        }
        false
    }
}

impl Action for SaveAction {
    fn execute(&self, buffer: &mut Buffer) {
        let path = self.path.to_owned();

        let _ = self
            .sender
            .send(Event::BufferSavingStarted(path.clone(), buffer.len()));

        // Détecter si le fichier doit être compressé en gzip
        let is_gzip = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("gz"))
            .unwrap_or(false);

        match File::create(&path) {
            Ok(file) => {
                if is_gzip {
                    let encoder = GzEncoder::new(file, Compression::default());
                    let mut writer = LineWriter::new(encoder);

                    if self.write_lines(buffer, &mut writer) {
                        // todo maybe an error ?
                        return;
                    }

                    // Finaliser l'encodeur gzip
                    match writer.into_inner() {
                        Ok(encoder) => {
                            if let Err(err) = encoder.finish() {
                                error!("Error finishing gzip compression: {err}");
                                let _ = self.sender.send(Event::BufferSaveFailed(path.clone()));
                                return;
                            }
                        }
                        Err(err) => {
                            error!("Error flushing writer: {err}");
                            let _ = self.sender.send(Event::BufferSaveFailed(path.clone()));
                            return;
                        }
                    }
                } else {
                    let mut writer = LineWriter::new(file);

                    if self.write_lines(buffer, &mut writer) {
                        // todo maybe an error ?
                        return;
                    }
                }

                // Fin de progression
                let _ = self.sender.send(Event::BufferSaved(path.clone()));
            }
            Err(err) => {
                error!("Unable to save file {path:?} becaues {err}");
                let _ = self.sender.send(Event::BufferSaveFailed(path.clone()));
            }
        }
    }
}
