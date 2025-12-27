// json_clean.rs
// JSON renaming/cleaning logic for MetaSort_v1.0.0 – Google Photos Takeout Organizer 

use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use std::io::{self, Write};
use crate::utils::{log_to_file, BufferedLogger};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::concurrency_config::get_concurrency_level;

pub fn ask_and_separate_whatsapp_screenshots(base_path: &str, separate_wa_sc: bool) {
    if !separate_wa_sc {
        return;
    }
    let logs_dir = Path::new(base_path).join("logs");
    log_to_file(&logs_dir, "media_cleaning.log", "User chose to separate WhatsApp and Screenshot images.");
    let whatsapp_patterns = vec![
        Regex::new(r"(?i)^(IMG-\d{8}-WA\d+|IMG-WA\d+|WA\d+|VID-\d{8}-WA\d+|VID-WA\d+|WhatsApp Image \d{4}-\d{2}-\d{2} at \d{2}\.\d{2}\.\d{2}|WhatsApp Video \d{4}-\d{2}-\d{2} at \d{2}\.\d{2}\.\d{2})").unwrap(),
    ];
    let screenshot_patterns = vec![
        Regex::new(r"(?i)^(Screenshot(_| )?\d{4}-\d{2}-\d{2}(-| )?\d{2}(-|\.|:)?\d{2}(-|\.|:)?\d{2}|Screenshot \(\d+\)|Screen Shot \d{4}-\d{2}-\d{2} at \d{2}\.\d{2}\.\d{2}|Screenshot_\d+|Screenshot_\d{8}-\d{6}|スクリーンショット|Снимок экрана|Captura de pantalla|Capture d'écran|Bildschirmfoto|Istantanea|Skjermbilde|Skärmbild|Ekran görüntüsü|Zrzut ekranu|PrtSc|Snip)").unwrap(),
    ];
    let other_images_dir = Path::new(base_path).join("Other Images");
    let whatsapp_dir = other_images_dir.join("Whatsapp");
    let screenshots_dir = other_images_dir.join("Screenshots");
    let _ = fs::create_dir_all(&whatsapp_dir);
    let _ = fs::create_dir_all(&screenshots_dir);
    let all_files: Vec<_> = WalkDir::new(base_path).into_iter().filter_map(Result::ok).filter(|e| e.path().is_file()).collect();
    let total = all_files.len();
    let processed = Arc::new(AtomicUsize::new(0));

    // Create thread-safe logger
    let logger = Arc::new(BufferedLogger::new(&logs_dir, "media_cleaning.log", 50));

    // Get concurrency configuration and create thread pool
    let concurrency_level = get_concurrency_level();
    let thread_count = concurrency_level.get_thread_count();

    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to create thread pool");

    // Parallel processing
    pool.install(|| {
        all_files.par_iter().for_each(|entry| {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // WhatsApp
            if whatsapp_patterns.iter().any(|re| re.is_match(filename)) {
                let dest = whatsapp_dir.join(filename);
                let _ = fs::rename(path, &dest);
                logger.log(&format!("Moved WhatsApp image {:?} to {:?}", path, dest));
                // Move .json if exists
                let json_path = path.with_extension(format!("{}.json", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
                if json_path.exists() {
                    let json_dest = whatsapp_dir.join(json_path.file_name().unwrap());
                    let _ = fs::rename(&json_path, &json_dest);
                    logger.log(&format!("Moved WhatsApp JSON {:?} to {:?}", json_path, json_dest));
                }
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                print_progress(current, total, path);
                return;
            }
            // Screenshot
            if screenshot_patterns.iter().any(|re| re.is_match(filename)) {
                let dest = screenshots_dir.join(filename);
                let _ = fs::rename(path, &dest);
                logger.log(&format!("Moved Screenshot image {:?} to {:?}", path, dest));
                // Move .json if exists
                let json_path = path.with_extension(format!("{}.json", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
                if json_path.exists() {
                    let json_dest = screenshots_dir.join(json_path.file_name().unwrap());
                    let _ = fs::rename(&json_path, &json_dest);
                    logger.log(&format!("Moved Screenshot JSON {:?} to {:?}", json_path, json_dest));
                }
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                print_progress(current, total, path);
            }
        }
        });
    });

    // Flush logger
    logger.flush();

    let final_processed = processed.load(Ordering::SeqCst);
    println!("\n[SUCCESS] WhatsApp/Screenshot separation complete! Processed {} files.", final_processed);
}

fn print_progress(done: usize, total: usize, file: &Path) {
    let percent = if total > 0 { (done * 100) / total } else { 100 };
    let filled = percent / 4;
    let empty = 25 - filled;
    let bar = format!("{}{}", "=".repeat(filled), "-".repeat(empty));
    let fname = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
    print!("\rCleaning: [{}] {}% ({}/{}) | {}", bar, percent, done, total, fname);
    let _ = io::stdout().flush();
    if done == total {
        println!();
    }
}

pub fn clean_json_filenames(base_path: &str) {
    let media_extensions = vec![
        // Images
        "jpg", "jpeg", "png", "webp", "heic", "heif", "bmp", "tiff", "gif", "avif", "jxl", "jfif",
        // Videos
        "mp4", "mov", "mkv", "avi", "webm", "3gp", "m4v", "mpg", "mpeg", "mts", "m2ts", "ts", "flv",
        "f4v", "wmv", "asf", "rm", "rmvb", "vob", "ogv", "mxf", "dv", "divx", "xvid"
    ];

    let temp_dir = Path::new(base_path).join("`MetaSort_temp");
    let _ = fs::create_dir_all(&temp_dir);
    let log_path = temp_dir.join("rename_log.txt");
    let mut log_file = fs::File::create(&log_path).expect("Failed to create log file");

    for entry in WalkDir::new(base_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lc = ext.to_lowercase();
                if media_extensions.contains(&ext_lc.as_str()) {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let parent = path.parent().unwrap_or(Path::new(""));
                    if let Ok(entries) = fs::read_dir(parent) {
                        for json_entry in entries {
                            if let Ok(json_file) = json_entry {
                                let json_path = json_file.path();
                                if json_path.is_file() {
                                    if let Some(json_name) = json_path.file_name().and_then(|n| n.to_str()) {
                                        if json_name.starts_with(filename) && json_name.ends_with(".json") && json_name != format!("{}.json", filename) {
                                            let new_json_path = parent.join(format!("{}.json", filename));
                                            if let Err(e) = fs::rename(&json_path, &new_json_path) {
                                                let _ = log_file.write_all(format!("❌ Failed to rename {:?} to {:?}: {}\n", json_path, new_json_path, e).as_bytes());
                                            } else {
                                                let _ = log_file.write_all(format!("✅ Renamed JSON {:?} to {:?}\n", json_path, new_json_path).as_bytes());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let summary = format!("\n[SUCCESS] JSON filename cleaning complete.\n");
    let _ = log_file.write_all(summary.as_bytes());
} 