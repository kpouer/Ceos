use crate::ceos::buffer::buffer::{Buffer, DEFAULT_GROUP_SIZE};
use crate::ceos::tools::misc_tool::{gzip_uncompressed_size_fast, is_gzip};
use crate::event::Event;
use crate::event::Event::{BufferLoading, BufferLoadingStarted};
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct BufferLoader {
    buffer: Buffer,
    sender: Sender<Event>,
}

impl BufferLoader {
    pub(crate) fn new_from_file(
        path: PathBuf,
        sender: Sender<Event>,
    ) -> Result<Buffer, std::io::Error> {
        let mut buffer_loader = Self {
            buffer: Buffer::new_with_group_size(sender.clone(), DEFAULT_GROUP_SIZE),
            sender,
        };
        buffer_loader.load_buffer(path)?;

        Ok(buffer_loader.buffer)
    }

    fn load_buffer(&mut self, path: PathBuf) -> Result<(), io::Error> {
        self.buffer.path = Some(path.clone());
        let file = File::open(&path)?;

        let mut buffer_reader = io::BufReader::new(file);

        let file_size = std::fs::metadata(&path)?.len() as usize;
        if is_gzip(&mut buffer_reader) {
            let file_size = match gzip_uncompressed_size_fast(&path) {
                Ok(size) => size as usize,
                Err(_) => file_size,
            };
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

    fn load_reader(
        &mut self,
        file_size: usize,
        buffer_reader: impl BufRead,
    ) -> Result<(), io::Error> {
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
