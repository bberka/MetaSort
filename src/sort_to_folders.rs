use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use chrono::Datelike;
use crate::csv_report;
use crate::utils::{log_to_file, BufferedLogger};
use std::io::Write;
use serde_json;
use crate::platform::get_exiftool_command;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::concurrency_config::get_concurrency_level;

/// Main function to organize files into folders by type and date.
pub fn sort_files_to_folders(input_dir: &Path, output_dir: &Path, failed_guess_paths: &Vec<PathBuf>, separate_wa_sc: bool) {
    let media_extensions = vec![
        // Images
        "jpg", "jpeg", "png", "webp", "heic", "heif", "bmp", "tiff", "gif", "avif", "jxl", "jfif",
        // Videos
        "mp4", "mov", "mkv", "avi", "webm", "3gp", "m4v", "mpg", "mpeg", "mts", "m2ts", "ts", "flv",
        "f4v", "wmv", "asf", "rm", "rmvb", "vob", "ogv", "mxf", "dv", "divx", "xvid"
    ];

    // Thread-safe collections for file info
    let photos_info = Arc::new(Mutex::new(Vec::new()));
    let videos_info = Arc::new(Mutex::new(Vec::new()));
    let unknown_info = Arc::new(Mutex::new(Vec::new()));
    let mkv_info = Arc::new(Mutex::new(Vec::new()));
    let failed_guess_info = Arc::new(Mutex::new(Vec::new()));

    let logs_dir = output_dir.join("Technical Files").join("logs");

    let all_files: Vec<_> = walkdir::WalkDir::new(input_dir).into_iter().filter_map(Result::ok).filter(|e| e.path().is_file()).collect();
    // Only count media files for progress
    let all_media_files: Vec<_> = all_files.iter().filter(|entry| {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            media_extensions.contains(&ext.as_str())
        } else {
            false
        }
    }).collect();
    let total = all_media_files.len();
    let processed = Arc::new(AtomicUsize::new(0));

    // Create thread-safe logger
    let logger = Arc::new(BufferedLogger::new(&logs_dir, "sorting.log", 50));

    // Get concurrency configuration and create thread pool
    let concurrency_level = get_concurrency_level();
    let thread_count = concurrency_level.get_thread_count();

    let pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to create thread pool");

    // Parallel processing
    pool.install(|| {
        all_media_files.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            // Use exiftool to get DateTimeOriginal, MIMEType, ImageSize
            let output = get_exiftool_command()
                .arg("-DateTimeOriginal")
                .arg("-MIMEType")
                .arg("-ImageSize")
                .arg("-FileType")
                .arg(path)
                .output();
            let mut date_str = String::new();
            let mut mime_type = String::new();
            let mut image_size = String::new();
            let mut file_type = String::new();
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if line.contains("Date/Time Original") {
                        date_str = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                    } else if line.contains("MIME Type") {
                        mime_type = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                    } else if line.contains("Image Size") {
                        image_size = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                    } else if line.contains("File Type") {
                        file_type = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                    }
                }
            }
            // If date_str is still empty, try to extract from JSON
            if date_str.is_empty() {
                let json_path = path.with_extension(format!("{}json", ext));
                if json_path.exists() {
                    if let Ok(json_str) = std::fs::read_to_string(&json_path) {
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            if let Some(ts) = json_val["photoTakenTime"]["timestamp"].as_str() {
                                if let Ok(timestamp) = ts.parse::<i64>() {
                                    use chrono::{TimeZone, Utc};
                                    let dt = Utc.timestamp_opt(timestamp, 0).unwrap();
                                    date_str = dt.format("%Y:%m:%d %H:%M:%S").to_string();
                                }
                            }
                        }
                    }
                }
            }
            let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let mut dest_folder = output_dir.join("Media Files");
            // WhatsApp/Screenshot detection
            let fname_lc = filename.to_lowercase();
            let is_wa = fname_lc.contains("wa") || fname_lc.contains("whatsapp");
            let is_sc = fname_lc.contains("screenshot");
            if separate_wa_sc && is_wa {
                dest_folder.push("Whatsapp");
                if let Some(dt) = parse_exif_date(&date_str) {
                    dest_folder.push(format!("{}", dt.year()));
                    dest_folder.push(format!("{}", month_name(dt.month())));
                }
                photos_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
            } else if separate_wa_sc && is_sc {
                dest_folder.push("Screenshots");
                if let Some(dt) = parse_exif_date(&date_str) {
                    dest_folder.push(format!("{}", dt.year()));
                    dest_folder.push(format!("{}", month_name(dt.month())));
                }
                photos_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
            } else if ext == "mkv" {
                let _file_category = "mkv_files".to_string();
                dest_folder.push("mkv_files");
                mkv_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
            } else if date_str.is_empty() {
                if failed_guess_paths.contains(&path.to_path_buf()) {
                    let _file_category = "Failed Filename Guess".to_string();
                    dest_folder.push("Unknown Time");
                    dest_folder.push("Failed Filename Guess");
                    failed_guess_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
                } else {
                    let _file_category = "Unknown Time".to_string();
                    dest_folder.push("Unknown Time");
                    unknown_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
                }
            } else if mime_type.starts_with("video") || ["mp4","mov","avi","webm","3gp","m4v","mpg","mpeg","mts","m2ts","ts","flv","f4v","wmv","asf","rm","rmvb","vob","ogv","mxf","dv","divx","xvid"].contains(&ext.as_str()) {
                // Video
                let _file_category = "Videos".to_string();
                dest_folder.push("Videos");
                if let Some(dt) = parse_exif_date(&date_str) {
                    dest_folder.push(format!("{}", dt.year()));
                    dest_folder.push(format!("{}", month_name(dt.month())));
                }
                videos_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
            } else {
                // Photo
                let _file_category = "Photos".to_string();
                dest_folder.push("Photos");
                if let Some(dt) = parse_exif_date(&date_str) {
                    dest_folder.push(format!("{}", dt.year()));
                    dest_folder.push(format!("{}", month_name(dt.month())));
                }
                photos_info.lock().unwrap().push((filename.clone(), file_type.clone(), date_str.clone(), image_size.clone(), human_readable_size(file_size), file_size));
            }
            // Create destination folder if needed
            let _ = fs::create_dir_all(&dest_folder);
            let dest_path = dest_folder.join(&filename);
            // Copy file
            match fs::copy(path, &dest_path) {
                Ok(_) => {
                    logger.log(&format!("Copied {:?} to {:?}", path.file_name().unwrap_or_default(), dest_path));
                }
                Err(e) => {
                    logger.log(&format!("Failed to copy {:?} to {:?}: {}", path.file_name().unwrap_or_default(), dest_path, e));
                }
            }
            let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
            print_progress(current, total);
        }
        });
    });

    // Flush logger
    logger.flush();

    // Unwrap Arc<Mutex<_>> to get the vectors back
    let photos_info = Arc::try_unwrap(photos_info).unwrap().into_inner().unwrap();
    let videos_info = Arc::try_unwrap(videos_info).unwrap().into_inner().unwrap();
    let unknown_info = Arc::try_unwrap(unknown_info).unwrap().into_inner().unwrap();
    let mkv_info = Arc::try_unwrap(mkv_info).unwrap().into_inner().unwrap();
    let failed_guess_info = Arc::try_unwrap(failed_guess_info).unwrap().into_inner().unwrap();
    // Write CSVs for each category in CSV Report folder
    let csv_report_folder = output_dir.join("Technical Files").join("CSV Report");
    let _ = fs::create_dir_all(&csv_report_folder);
    csv_report::write_csv_report(&csv_report_folder, &photos_info, "photos.csv");
    csv_report::write_csv_report(&csv_report_folder, &videos_info, "videos.csv");
    csv_report::write_csv_report(&csv_report_folder, &unknown_info, "unknown_time.csv");
    csv_report::write_csv_report(&csv_report_folder, &mkv_info, "mkv_files.csv");
    csv_report::write_csv_report(&csv_report_folder, &failed_guess_info, "failed_filename_guess.csv");
    log_to_file(&logs_dir, "sorting.log", "CSV reports written for Photos, Videos, Unknown Time, and mkv_files.");
    let final_processed = processed.load(Ordering::SeqCst);
    println!("\n[SUCCESS] Sorting complete! Sorted {} files.", final_processed);
    println!("\nCSV files are added in: {}\nPlease keep this folder safe for future use!", csv_report_folder.display());

    let failed_guess_folder = output_dir.join("Media Files").join("Unknown Time").join("Failed Filename Guess");
    let _ = fs::create_dir_all(&failed_guess_folder);
    for path in failed_guess_paths {
        if let Some(filename) = path.file_name() {
            let dest = failed_guess_folder.join(filename);
            let _ = fs::rename(path, &dest);
            log_to_file(&logs_dir, "sorting.log", &format!("Moved failed guess file {:?} to {:?}", path, dest));
        }
    }
}

fn parse_exif_date(date_str: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S").ok()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn print_progress(done: usize, total: usize) {
    let percent = if total > 0 { (done * 100) / total } else { 100 };
    let filled = percent / 4;
    let empty = 25 - filled;
    let bar = format!("{}{}", "=".repeat(filled), "-".repeat(empty));
    print!("\rSorting: [{}] {}% ({} / {})", bar, percent, done, total);
    let _ = io::stdout().flush();
    if done == total {
        println!();
    }
}

fn human_readable_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match size {
        s if s >= GB => format!("{:.2} GB", s as f64 / GB as f64),
        s if s >= MB => format!("{:.2} MB", s as f64 / MB as f64),
        s if s >= KB => format!("{:.2} KB", s as f64 / KB as f64),
        _ => format!("{} B", size),
    }
}

