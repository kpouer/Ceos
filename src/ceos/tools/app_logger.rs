use log::{Level, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

const LOG_FILE: &str = "activity.log";

pub struct AppLogger {
    file: Mutex<Option<File>>,
}

impl Default for AppLogger {
    fn default() -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE)
            .ok();

        AppLogger {
            file: Mutex::new(file),
        }
    }
}

impl AppLogger {
    pub fn init() {
        let logger = AppLogger::default();
        log::set_boxed_logger(Box::new(logger)).expect("Could not set logger");
        log::set_max_level(log::LevelFilter::Info);
    }

    fn write_to_file(&self, message: &str) {
        let mut file_lock = self.file.lock().unwrap();

        if let Some(file) = file_lock.as_mut() {
            let _ = writeln!(file, "{}", message);
        }
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("[{}] {}", record.level(), record.args());

            // Enregistrement dans le fichier
            self.write_to_file(&message);
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_logger_writing() {
        let log_file = "activity.log";

        // Nettoyage initial
        let _ = fs::remove_file(log_file);

        let logger = AppLogger {
            file: Mutex::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .ok(),
            ),
        };

        logger.write_to_file("test message");

        let mut content = String::new();
        File::open(log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert!(content.contains("test message"));

        let _ = fs::remove_file(log_file);
    }
}
