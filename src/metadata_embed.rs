// metadata_embed.rs
// Embedding metadata logic for MetaSort_v1.0.0 – Google Photos Takeout Organizer

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use crate::metadata_extraction::MediaMetadata;
use crate::filename_date_guess::extract_date_from_filename;
use crate::utils::BufferedLogger;
use crate::platform::get_exiftool_command;

#[derive(Debug, Clone, Copy)]
enum ConcurrencyLevel {
    Slow,
    Medium,
    Unlimited,
}

impl ConcurrencyLevel {
    fn get_thread_count(&self) -> usize {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        match self {
            ConcurrencyLevel::Slow => num_cpus.min(4).max(2),
            ConcurrencyLevel::Medium => (num_cpus / 2).max(2),
            ConcurrencyLevel::Unlimited => num_cpus,
        }
    }

    fn from_user_input(input: &str) -> Option<Self> {
        match input.trim() {
            "1" => Some(ConcurrencyLevel::Slow),
            "2" => Some(ConcurrencyLevel::Medium),
            "3" => Some(ConcurrencyLevel::Unlimited),
            _ => None,
        }
    }
}

fn process_single_metadata(
    meta: &MediaMetadata,
    use_filename: bool,
    logger: &Arc<BufferedLogger>,
    processed: &Arc<AtomicUsize>,
    total: usize,
) {
    let mut args = Vec::new();
    let filename = meta.media_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let parent = meta.media_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("");
    let is_wa = parent.eq_ignore_ascii_case("Whatsapp");
    let is_sc = parent.eq_ignore_ascii_case("Screenshots");
    let mut used = "metadata";
    let mut date_to_embed = meta.exif_date.clone();

    if use_filename && (is_wa || is_sc) {
        if let Some(date) = extract_date_from_filename(filename) {
            date_to_embed = Some(date);
            used = "filename";
        }
    }

    if date_to_embed.is_none() {
        used = "metadata (fallback)";
    }

    if let Some(ref date) = date_to_embed {
        if meta.media_path.extension().map(|e| e.to_ascii_lowercase()) == Some("png".into()) {
            args.push(format!("-XMP:DateTimeOriginal={}", date));
        } else {
            args.push(format!("-DateTimeOriginal={}", date));
        }
    }

    if let (Some(lat), Some(lon)) = (meta.gps_latitude, meta.gps_longitude) {
        args.push(format!("-GPSLatitude={}", lat));
        args.push(format!("-GPSLongitude={}", lon));
    }

    if let Some(alt) = meta.gps_altitude {
        args.push(format!("-GPSAltitude={}", alt));
    }

    if let Some(ref make) = meta.camera_make {
        args.push(format!("-Make={}", make));
    }

    if let Some(ref model) = meta.camera_model {
        args.push(format!("-Model={}", model));
    }

    args.push("-overwrite_original".to_string());
    args.push(meta.media_path.to_string_lossy().to_string());

    let log_msg = format!(
        "File: {:?}, Used: {}, Date: {:?}, Lat: {:?}, Lon: {:?}, Alt: {:?}, Make: {:?}, Model: {:?}",
        meta.media_path.file_name().unwrap_or_default(),
        used,
        date_to_embed,
        meta.gps_latitude,
        meta.gps_longitude,
        meta.gps_altitude,
        meta.camera_make,
        meta.camera_model
    );

    let status = get_exiftool_command()
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(status) if status.success() => {
            logger.log(&format!("[SUCCESS] Embedded metadata. {}", log_msg));
        }
        Ok(_) => {
            logger.log(&format!("[FAILED] Failed to embed metadata. {}", log_msg));
        }
        Err(_) => {
            logger.log(&format!("[ERROR] Error running exiftool. {}", log_msg));
        }
    }

    let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
    print_progress(current, total);
}

pub fn embed_metadata_all(metadata_list: &[MediaMetadata], log_dir: &Path) {
    let logs_dir = log_dir.join("logs");
    let log_path = logs_dir.join("metadata_embedding.log");
    let _ = fs::create_dir_all(&logs_dir);
    let _log_file = File::create(&log_path).expect("Failed to create log file");

    // Prompt 1: Metadata vs Filename choice
    println!("\nDo you want to embed date/time for WhatsApp & Screenshot images based on their:");
    println!("1. Metadata");
    println!("2. Filename");
    println!("Enter 1 or 2:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let use_filename = matches!(input.trim(), "2");

    // Prompt 2: Concurrency level choice
    println!("\nSelect processing speed (concurrency level):");
    println!("1. Slow (2-4 threads) - For limited resources");
    println!("2. Medium (half CPU cores) - RECOMMENDED for balanced performance");
    println!("3. Unlimited (all CPU cores) - Maximum speed, high resource usage");
    println!("Enter 1, 2, or 3:");

    let mut concurrency_input = String::new();
    io::stdin().read_line(&mut concurrency_input).expect("Failed to read line");

    let concurrency_level = ConcurrencyLevel::from_user_input(&concurrency_input)
        .unwrap_or_else(|| {
            println!("[WARNING] Invalid input, defaulting to Medium (recommended)");
            ConcurrencyLevel::Medium
        });

    let thread_count = concurrency_level.get_thread_count();
    println!("Using {} threads for parallel processing\n", thread_count);

    // Configure Rayon thread pool
    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to create thread pool");

    // Thread-safe progress counter
    let total = metadata_list.len();
    let processed = Arc::new(AtomicUsize::new(0));

    // Thread-safe logger
    let logger = Arc::new(BufferedLogger::new(&logs_dir, "metadata_embedding.log", 50));

    // Parallel processing with Rayon
    pool.install(|| {
        metadata_list.par_iter().for_each(|meta| {
            process_single_metadata(
                meta,
                use_filename,
                &logger,
                &processed,
                total,
            );
        });
    });

    // Ensure all logs are written
    logger.flush();

    let final_processed = processed.load(Ordering::SeqCst);
    println!("\n[SUCCESS] Metadata embedding complete! Embedded metadata for {} files. Log: {:?}", final_processed, log_path);
}

fn print_progress(done: usize, total: usize) {
    let percent = if total > 0 { (done * 100) / total } else { 100 };
    let filled = percent / 4;
    let empty = 25 - filled;
    let bar = format!("{}{}", "=".repeat(filled), "-".repeat(empty));
    print!("\rEmbedding metadata: [{}] {}% ({} / {})", bar, percent, done, total);
    let _ = std::io::stdout().flush();
    if done == total {
        println!();
    }
} 