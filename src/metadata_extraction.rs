use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde_json::Value;
use chrono::{TimeZone, Utc};
use crate::utils::{log_to_file, BufferedLogger};
use std::io;
use std::io::Write;
use crate::filename_date_guess::extract_date_from_filename;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::concurrency_config::get_concurrency_level;

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub media_path: PathBuf,
    pub _json_path: PathBuf,
    pub exif_date: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

pub fn extract_metadata(base_path: &str) -> (Vec<MediaMetadata>, Vec<PathBuf>) {
    let mut media_json_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut all_media_files: Vec<PathBuf> = Vec::new();
    let media_extensions = vec![
        // Images
        "jpg", "jpeg", "png", "webp", "heic", "heif", "bmp", "tiff", "gif", "avif", "jxl", "jfif",
        "raw", "cr2", "nef", "orf", "sr2", "arw", "dng", "pef", "raf", "rw2", "srw", "3fr", "erf",
        "k25", "kdc", "mef", "mos", "mrw", "nrw", "srf", "x3f", "svg", "ico", "psd", "ai", "eps",
        // Videos
        "mp4", "mov", "mkv", "avi", "webm", "3gp", "m4v", "mpg", "mpeg", "mts", "m2ts", "ts", "flv",
        "f4v", "wmv", "asf", "rm", "rmvb", "vob", "ogv", "mxf", "dv", "divx", "xvid"
    ];

    let logs_dir = Path::new(base_path).join("logs");

    // Find all media files and their matching .json
    for entry in WalkDir::new(base_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lc = ext.to_lowercase();
                if media_extensions.contains(&ext_lc.as_str()) {
                    all_media_files.push(path.to_path_buf());
                    let json_path = path.with_extension(format!("{}.json", ext_lc));
                    let json_path_alt = path.with_extension(format!("{}", "json"));
                    // Try both: IMG_001.JPG.json and IMG_001.jpg.json
                    if json_path.exists() {
                        media_json_pairs.push((path.to_path_buf(), json_path));
                    } else if json_path_alt.exists() {
                        media_json_pairs.push((path.to_path_buf(), json_path_alt));
                    }
                }
            }
        }
    }

    let paired_media: Vec<PathBuf> = media_json_pairs.iter().map(|(m, _)| m.clone()).collect();
    let unpaired_media: Vec<PathBuf> = all_media_files.into_iter().filter(|m| !paired_media.contains(m)).collect();

    // Thread-safe collection for metadata
    let metadata_list = Arc::new(Mutex::new(Vec::new()));
    let total = media_json_pairs.len();
    let processed = Arc::new(AtomicUsize::new(0));

    // Create thread-safe logger
    let logger = Arc::new(BufferedLogger::new(&logs_dir, "metadata_extraction.log", 50));

    // Get concurrency configuration and create thread pool
    let concurrency_level = get_concurrency_level();
    let thread_count = concurrency_level.get_thread_count();

    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to create thread pool");

    // Parallel processing of paired media/JSON files
    pool.install(|| {
        media_json_pairs.par_iter().for_each(|(media_path, json_path)| {
        let json_str = match fs::read_to_string(json_path) {
            Ok(s) => s,
            Err(e) => {
                logger.log(&format!("Failed to read JSON for {:?}: {}", json_path, e));
                return;
            }
        };
        let v: Value = match serde_json::from_str(&json_str) {
            Ok(val) => val,
            Err(e) => {
                logger.log(&format!("Failed to parse JSON for {:?}: {}", json_path, e));
                return;
            }
        };
        // Extract timestamp and convert to EXIF format (original date only)
        let exif_date = v["photoTakenTime"]["timestamp"].as_str().and_then(|ts| {
            ts.parse::<i64>().ok().map(|timestamp| {
                let dt = Utc.timestamp_opt(timestamp, 0).unwrap();
                dt.format("%Y:%m:%d %H:%M:%S").to_string()
            })
        });
        // Extract GPS
        let gps_latitude = v["geoData"]["latitude"].as_f64()
            .or_else(|| v["geoDataExif"]["latitude"].as_f64());
        let gps_longitude = v["geoData"]["longitude"].as_f64()
            .or_else(|| v["geoDataExif"]["longitude"].as_f64());
        let gps_altitude = v["geoData"]["altitude"].as_f64()
            .or_else(|| v["geoDataExif"]["altitude"].as_f64());
        // Camera make/model
        let camera_make = v["cameraMake"].as_str().map(|s| s.to_string());
        let camera_model = v["cameraModel"].as_str().map(|s| s.to_string());

        metadata_list.lock().unwrap().push(MediaMetadata {
            media_path: media_path.clone(),
            _json_path: json_path.clone(),
            exif_date,
            gps_latitude,
            gps_longitude,
            gps_altitude,
            camera_make,
            camera_model,
        });
        let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
        print_progress(current, total);
        });
    });

    // Flush logger
    logger.flush();

    // Unwrap Arc<Mutex<_>> to get the vector back
    let mut metadata_list = Arc::try_unwrap(metadata_list).unwrap().into_inner().unwrap();
    let mut failed_guess_paths = Vec::new();
    // Handle unpaired media
    if !unpaired_media.is_empty() {
        println!(
            "\n[WARNING] No .json found for {} out of {} files ({}%).\nWhat should MetaSort do?\n1. Skip and move to 'Unknown Time'\n2. Try to guess timestamp from filename\nEnter 1 or 2:",
            unpaired_media.len(), paired_media.len() + unpaired_media.len(), (unpaired_media.len() * 100) / (paired_media.len() + unpaired_media.len())
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let guess = input.trim() == "2";
        for media_path in unpaired_media {
            let filename = media_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let exif_date = if guess {
                if let Some(date) = extract_date_from_filename(filename) {
                    log_to_file(&logs_dir, "metadata_extraction.log", &format!("Guessed date from filename for {:?}: {}", filename, date));
                    Some(date)
                } else {
                    log_to_file(&logs_dir, "metadata_extraction.log", &format!("Could not guess date from filename for {:?}", filename));
                    failed_guess_paths.push(media_path.clone());
                    None
                }
            } else {
                log_to_file(&logs_dir, "metadata_extraction.log", &format!("No JSON for {:?}, moved to Unknown Time", filename));
                None
            };
            metadata_list.push(MediaMetadata {
                media_path: media_path.clone(),
                _json_path: PathBuf::new(),
                exif_date,
                gps_latitude: None,
                gps_longitude: None,
                gps_altitude: None,
                camera_make: None,
                camera_model: None,
            });
        }
    }
    (metadata_list, failed_guess_paths)
}

fn print_progress(done: usize, total: usize) {
    let percent = if total > 0 { (done * 100) / total } else { 100 };
    let filled = percent / 4;
    let empty = 25 - filled;
    let bar = format!("{}{}", "=".repeat(filled), "-".repeat(empty));
    print!("\rExtracting metadata: [{}] {}% ({} / {})", bar, percent, done, total);
    let _ = std::io::stdout().flush();
    if done == total {
        println!();
    }
} 