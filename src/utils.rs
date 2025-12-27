// utils.rs
// Utility/helper functions for MetaSort_v1.0.0 – Google Photos Takeout Organizer

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::collections::VecDeque;
use chrono::Local;

/// Appends a timestamped log entry to a log file in the logs folder inside the given directory.
pub fn log_to_file(log_dir: &Path, log_name: &str, message: &str) {
    let _ = create_dir_all(log_dir);
    let log_path = log_dir.join(log_name);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Unable to open log file");
    let now = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    let _ = writeln!(file, "{} {}", now, message);
}

/// Thread-safe buffered logger for parallel processing
pub struct BufferedLogger {
    log_dir: PathBuf,
    log_name: String,
    buffer: Mutex<VecDeque<String>>,
    batch_size: usize,
}

impl BufferedLogger {
    pub fn new(log_dir: &Path, log_name: &str, batch_size: usize) -> Self {
        let _ = create_dir_all(log_dir);
        Self {
            log_dir: log_dir.to_path_buf(),
            log_name: log_name.to_string(),
            buffer: Mutex::new(VecDeque::new()),
            batch_size,
        }
    }

    pub fn log(&self, message: &str) {
        let now = Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let formatted = format!("{} {}", now, message);

        let mut buffer = self.buffer.lock().unwrap();
        buffer.push_back(formatted);

        // Flush if batch size reached
        if buffer.len() >= self.batch_size {
            self.flush_internal(&mut buffer);
        }
    }

    fn flush_internal(&self, buffer: &mut VecDeque<String>) {
        if buffer.is_empty() {
            return;
        }

        let log_path = self.log_dir.join(&self.log_name);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            for msg in buffer.drain(..) {
                let _ = writeln!(file, "{}", msg);
            }
        }
    }

    pub fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        self.flush_internal(&mut buffer);
    }
}

impl Drop for BufferedLogger {
    fn drop(&mut self) {
        self.flush();
    }
} 