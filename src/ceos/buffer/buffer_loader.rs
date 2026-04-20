use crate::ceos::buffer::buffer::{Buffer, DEFAULT_GROUP_SIZE};
use crate::ceos::tools::misc_tool::{gzip_uncompressed_size_fast, is_gzip};
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io;
use std::io::{BufRead, Error};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct BufferLoader {
    buffer: Buffer,
    sender: Sender<Event>,
}

impl BufferLoader {
    pub(crate) fn new_from_file(path: PathBuf, sender: Sender<Event>) -> Result<Buffer, Error> {
        let mut buffer_loader = Self {
            buffer: Buffer::new_with_group_size(sender.clone(), DEFAULT_GROUP_SIZE),
            sender,
        };
        buffer_loader.load_buffer(path)?;

        Ok(buffer_loader.buffer)
    }

    fn load_buffer(&mut self, path: PathBuf) -> Result<(), Error> {
        self.buffer.path = Some(path.clone());
        let file = File::open(&path)?;

        let mut buffer_reader = io::BufReader::new(file);

        let file_size = std::fs::metadata(&path)?.len();
        if is_gzip(&mut buffer_reader) {
            let file_size = match gzip_uncompressed_size_fast(&path) {
                Ok(size) => size as u64,
                Err(_) => file_size,
            }
            .max(file_size);
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            let decoder = GzDecoder::new(buffer_reader);
            let mut buffer_reader = io::BufReader::new(decoder);
            self.load_reader(file_size, &mut buffer_reader)?;
        } else {
            let _ = self
                .sender
                .send(BufferLoadingStarted(path.clone(), file_size));
            self.load_reader(file_size, &mut buffer_reader)?;
        }

        Ok(())
    }

    fn load_reader(&mut self, file_size: u64, buffer_reader: impl BufRead) -> Result<(), Error> {
        let mut start = Instant::now();
        for line_text in buffer_reader.lines() {
            self.buffer.push_line(line_text?);
            if start.elapsed() > Duration::from_millis(50) {
                let path = self.buffer.path.clone().expect("buffer has no path");
                let _ = self
                    .sender
                    .send(BufferLoading(path, self.buffer.len(), file_size));
                start = Instant::now();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ceos::buffer::buffer_loader::BufferLoader;
    use std::path::PathBuf;

    #[test]
    fn new_from_file_loads_cargo_toml() {
        let (sender, _) = std::sync::mpsc::channel();
        let path = PathBuf::from("Cargo.toml");
        let mut buffer =
            BufferLoader::new_from_file(path, sender).expect("Failed to load Cargo.toml");

        assert!(buffer.line_count() > 0);
        let first_line = buffer.line_text(0);
        assert!(first_line.contains("[package]"));
        buffer.compress_all_groups();
    }
}
