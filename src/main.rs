// main.rs
// Entry point for MetaSort_v1.0.0 – Google Photos Takeout Organizer
// Modified by Berkay (2025): Enhanced path handling and CLI support
// - Supports paths with spaces, quotes, UNC paths, and network drives
// - CLI mode with clap alongside interactive mode
// - Enhanced error handling with actionable messages

mod media_cleaning;
mod metadata_extraction;
mod metadata_embed;
mod sort_to_folders;
mod html_report;
mod utils;
mod csv_report;
mod filename_date_guess;
mod platform;
mod ui;
mod concurrency_config;
mod path_utils;

use std::io;
use std::path::{Path, PathBuf};
use std::fs;
use walkdir;
use fs_extra;
use clap::Parser;
use crate::platform::{is_exiftool_available, get_installation_instructions};
use crate::ui::MetaSortUI;
use crate::path_utils::{PathType, PathError};

fn get_folder_size(path: &str) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
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

/// MetaSort - Google Photos Takeout Organizer with Metadata Embedding
#[derive(Parser, Debug)]
#[command(name = "MetaSort")]
#[command(author = "Sanmith S., Berkay")]
#[command(version = "1.1.0-bberka")]
#[command(about = "Process and organize Google Photos Takeout exports with embedded metadata", long_about = None)]
struct Cli {
    /// Input directory path (Google Photos Takeout folder)
    #[arg(short = 'i', long = "input", value_name = "INPUT_DIR")]
    input_dir: Option<String>,

    /// Output directory path (where MetaSort will create working folders)
    #[arg(short = 'o', long = "output", value_name = "OUTPUT_DIR")]
    output_dir: Option<String>,

    /// Concurrency level: slow, medium, unlimited
    #[arg(short = 'c', long = "concurrency", value_name = "LEVEL")]
    concurrency: Option<String>,

    /// Separate WhatsApp and Screenshot images into their own folders
    #[arg(long = "separate-whatsapp-screenshots")]
    separate_wa_sc: bool,

    /// Skip confirmation prompts (auto-confirm)
    #[arg(short = 'y', long = "yes")]
    auto_confirm: bool,
}

fn main() {
    let cli = Cli::parse();

    MetaSortUI::print_header();

    // Check if exiftool is available
    if !is_exiftool_available() {
        MetaSortUI::print_error("ExifTool is not installed or not found in PATH!");
        println!("{}", get_installation_instructions());
        println!("\nPress Enter to exit...");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        return;
    }

    MetaSortUI::print_success("ExifTool found and ready!");

    // Determine if running in interactive or CLI mode
    let is_interactive = cli.input_dir.is_none() || cli.output_dir.is_none();

    if is_interactive {
        run_interactive_mode(cli);
    } else {
        run_cli_mode(cli);
    }
}

fn run_interactive_mode(cli: Cli) {
    // Initialize concurrency (or use CLI-provided level if available)
    let _concurrency_level = if let Some(level_str) = cli.concurrency {
        concurrency_config::parse_concurrency_level(&level_str).unwrap_or_else(|| {
            MetaSortUI::print_warning("Invalid concurrency level specified, using interactive selection");
            concurrency_config::initialize_concurrency()
        })
    } else {
        concurrency_config::initialize_concurrency()
    };

    println!("(Supports paths with spaces, quoted paths, and UNC network paths like \\\\server\\share)");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // Validate input directory with enhanced path handling
    let input_path = match path_utils::validate_input_dir(&input) {
        Ok(path) => path,
        Err(e) => {
            MetaSortUI::print_error(&format!("Invalid input directory: {}", e));
            match e {
                PathError::UNCNotSupported(_) => {
                    println!("\nUNC path troubleshooting:");
                    println!("1. Verify the network path is accessible in File Explorer");
                    println!("2. Ensure you have proper network credentials");
                    println!("3. Try mapping the network drive and using the mapped drive letter instead");
                    println!("   Example: Instead of \\\\server\\share, use Z:\\ (after mapping)");
                },
                PathError::DoesNotExist(_) => {
                    println!("\nPath does not exist. Please check the path and try again.");
                },
                PathError::PermissionDenied(_) => {
                    println!("\nPermission denied. Run MetaSort with administrator privileges or check folder permissions.");
                },
                _ => {}
            }
            return;
        }
    };

    // Check for network paths and show warning
    let path_type = path_utils::detect_path_type(&input_path);
    if let Some(warning) = path_utils::get_network_path_warning(path_type) {
        MetaSortUI::print_warning(&warning);
        if !cli.auto_confirm {
            println!("Do you want to continue? (y/n)");
            let mut response = String::new();
            io::stdin().read_line(&mut response).expect("Failed to read line");
            if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
                MetaSortUI::print_info("Operation cancelled by user.");
                return;
            }
        }
    }

    // Calculate input folder size and prompt for required space
    let input_dir_str = input_path.to_str().expect("Invalid UTF-8 in path");
    let folder_size = get_folder_size(input_dir_str);
    let required_space = folder_size * 3;
    MetaSortUI::print_info(&format!("Input folder size: {}", human_readable_size(folder_size)));
    MetaSortUI::print_info(&format!("Recommended free space: {}", human_readable_size(required_space)));

    if !cli.auto_confirm {
        println!("Continue? (y/n)");
        let mut cont = String::new();
        io::stdin().read_line(&mut cont).expect("Failed to read line");
        if !matches!(cont.trim().to_lowercase().as_str(), "y" | "yes") {
            MetaSortUI::print_warning("Aborted by user.");
            return;
        }
    }

    // Prompt for output folder
    println!("\nPlease specify the output folder where MetaSort should work (originals will be untouched):");
    let mut output = String::new();
    io::stdin().read_line(&mut output).expect("Failed to read line");

    // Validate output directory - ask before creating if doesn't exist
    let output_path = match path_utils::validate_output_dir(&output, false) {
        Ok(path) => path,
        Err(PathError::DoesNotExist(p)) => {
            println!("\nOutput directory does not exist: {}", p);
            println!("Create this directory? (y/n)");
            let mut create_response = String::new();
            io::stdin().read_line(&mut create_response).expect("Failed to read line");
            if matches!(create_response.trim().to_lowercase().as_str(), "y" | "yes") {
                match path_utils::validate_output_dir(&output, true) {
                    Ok(path) => {
                        MetaSortUI::print_success(&format!("Created output directory: {}", path.display()));
                        path
                    },
                    Err(e) => {
                        MetaSortUI::print_error(&format!("Failed to create output directory: {}", e));
                        return;
                    }
                }
            } else {
                MetaSortUI::print_warning("Aborted by user.");
                return;
            }
        },
        Err(e) => {
            MetaSortUI::print_error(&format!("Invalid output directory: {}", e));
            return;
        }
    };

    let temp_dir = output_path.join("MetaSort_temp");

    // Copy input folder to MetaSort_temp in output directory
    MetaSortUI::print_section_header("Copying Files");
    MetaSortUI::print_info("Copying input folder to working directory...");

    let mut ui = MetaSortUI::new();
    let total_files = count_files_in_directory(input_dir_str);
    ui.start_main_progress(total_files as u64, "Copying files");

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.copy_inside = true;

    // Enhanced error handling for copy operation
    match fs_extra::dir::copy(input_dir_str, &temp_dir, &copy_options) {
        Ok(_) => {
            ui.finish_progress("Copy complete!");
            MetaSortUI::print_success(&format!("All processing will happen in: {}", temp_dir.display()));
        },
        Err(e) => {
            ui.finish_progress("Copy failed!");
            MetaSortUI::print_error(&format!("Failed to copy input folder to output working directory: {}", e));
            println!("\nPossible causes:");
            println!("1. Insufficient disk space in output directory");
            println!("2. Permission denied - try running with administrator privileges");
            println!("3. Output directory is on read-only media");
            println!("4. File in use by another application");

            // Check if it's a network path issue
            if path_utils::detect_path_type(&temp_dir) == PathType::NetworkUNC {
                println!("5. Network connectivity issues - verify network path is accessible");
            }

            return;
        }
    }

    // 1b. Ask if WhatsApp/Screenshots should be separated
    let separate_wa_sc = if cli.separate_wa_sc {
        true
    } else {
        println!("\nDo you want to separate WhatsApp and Screenshot images? (y/n)");
        let mut wa_sc_input = String::new();
        io::stdin().read_line(&mut wa_sc_input).expect("Failed to read line");
        matches!(wa_sc_input.trim().to_lowercase().as_str(), "y" | "yes")
    };

    // Call the shared processing function
    process_takeout(&temp_dir, &output_path, separate_wa_sc);
}

fn run_cli_mode(cli: Cli) {
    // Parse concurrency level
    let concurrency_level = if let Some(level_str) = cli.concurrency {
        concurrency_config::parse_concurrency_level(&level_str).unwrap_or_else(|| {
            MetaSortUI::print_warning("Invalid concurrency level, defaulting to Medium");
            concurrency_config::ConcurrencyLevel::Medium
        })
    } else {
        // Default to Medium for CLI mode
        concurrency_config::ConcurrencyLevel::Medium
    };

    // Set concurrency in global config
    concurrency_config::set_concurrency_level(concurrency_level);
    let thread_count = concurrency_level.get_thread_count();
    println!("Using {} threads for all parallel operations\n", thread_count);

    let input_dir = cli.input_dir.unwrap();
    let output_dir = cli.output_dir.unwrap();

    // Validate input directory
    let input_path = match path_utils::validate_input_dir(&input_dir) {
        Ok(path) => path,
        Err(e) => {
            MetaSortUI::print_error(&format!("Invalid input directory: {}", e));
            std::process::exit(1);
        }
    };

    // Check for network paths and show warning
    let path_type = path_utils::detect_path_type(&input_path);
    if let Some(warning) = path_utils::get_network_path_warning(path_type) {
        MetaSortUI::print_warning(&warning);
    }

    // Validate output directory (auto-create in CLI mode)
    let output_path = match path_utils::validate_output_dir(&output_dir, true) {
        Ok(path) => path,
        Err(e) => {
            MetaSortUI::print_error(&format!("Invalid output directory: {}", e));
            std::process::exit(1);
        }
    };

    // Size check
    let input_dir_str = input_path.to_str().expect("Invalid UTF-8 in path");
    let folder_size = get_folder_size(input_dir_str);
    let required_space = folder_size * 3;
    MetaSortUI::print_info(&format!("Input folder size: {}", human_readable_size(folder_size)));
    MetaSortUI::print_info(&format!("Recommended free space: {}", human_readable_size(required_space)));

    if !cli.auto_confirm {
        println!("Continue? (y/n)");
        let mut cont = String::new();
        io::stdin().read_line(&mut cont).expect("Failed to read line");
        if !matches!(cont.trim().to_lowercase().as_str(), "y" | "yes") {
            MetaSortUI::print_warning("Aborted by user.");
            return;
        }
    }

    let temp_dir = output_path.join("MetaSort_temp");

    // Copy input folder
    MetaSortUI::print_section_header("Copying Files");
    MetaSortUI::print_info("Copying input folder to working directory...");

    let mut ui = MetaSortUI::new();
    let total_files = count_files_in_directory(input_dir_str);
    ui.start_main_progress(total_files as u64, "Copying files");

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.copy_inside = true;

    match fs_extra::dir::copy(input_dir_str, &temp_dir, &copy_options) {
        Ok(_) => {
            ui.finish_progress("Copy complete!");
            MetaSortUI::print_success(&format!("All processing will happen in: {}", temp_dir.display()));
        },
        Err(e) => {
            ui.finish_progress("Copy failed!");
            MetaSortUI::print_error(&format!("Failed to copy: {}", e));
            std::process::exit(1);
        }
    }

    // Call the shared processing function
    process_takeout(&temp_dir, &output_path, cli.separate_wa_sc);
}

fn process_takeout(temp_dir: &Path, output_dir: &Path, separate_wa_sc: bool) {
    // 1. Clean and pair media files with their JSONs (fix weird JSON names)
    MetaSortUI::print_section_header("Cleaning and Pairing Files");
    MetaSortUI::print_info("Cleaning and pairing media files with JSONs...");
    media_cleaning::clean_json_filenames(temp_dir.to_str().unwrap());
    MetaSortUI::print_success("JSON filename cleaning and pairing complete!");

    if separate_wa_sc {
        MetaSortUI::print_success("WhatsApp and Screenshot images will be sorted into their own folders by year/month.");
    } else {
        MetaSortUI::print_info("WhatsApp and Screenshot images will be treated as regular photos.");
    }
    media_cleaning::ask_and_separate_whatsapp_screenshots(temp_dir.to_str().unwrap(), separate_wa_sc);

    // 2. Extract metadata from JSON and embed into media files
    MetaSortUI::print_section_header("Metadata Extraction and Embedding");
    MetaSortUI::print_info("Extracting metadata from JSON and embedding into media files...");
    let (metadata, failed_guess_paths) = metadata_extraction::extract_metadata(temp_dir.to_str().unwrap());
    metadata_embed::embed_metadata_all(&metadata, temp_dir);
    MetaSortUI::print_success("Metadata extraction and embedding complete!");

    // 3. Sort files using the embedded metadata (DateTimeOriginal)
    MetaSortUI::print_section_header("Sorting Files");
    MetaSortUI::print_info("Sorting files using embedded metadata...");
    let final_output_dir = output_dir.join("MetaSort_Output");
    sort_to_folders::sort_files_to_folders(temp_dir, &final_output_dir, &failed_guess_paths, separate_wa_sc);
    MetaSortUI::print_success("All done! Check your output and logs for details.");

    // 4. Move technical folders into Technical Files
    let technical_dir = final_output_dir.join("Technical Files");
    let _ = fs::create_dir_all(&technical_dir);
    for folder in &["CSV Report", "Json Files", "logs"] {
        let src = final_output_dir.join(folder);
        let dst = technical_dir.join(folder);
        if src.exists() {
            let _ = fs_extra::dir::move_dir(&src, &dst, &fs_extra::dir::CopyOptions::new());
        }
    }

    // 5. HTML summary report
    let photos = count_files(&final_output_dir.join("Media Files/Photos"));
    let videos = count_files(&final_output_dir.join("Media Files/Videos"));
    let whatsapp = count_files(&final_output_dir.join("Media Files/Whatsapp"));
    let screenshots = count_files(&final_output_dir.join("Media Files/Screenshots"));
    let unknown = count_files(&final_output_dir.join("Media Files/Unknown Time"));
    let mkv = count_files(&final_output_dir.join("Media Files/mkv_files"));
    let total = photos + videos + whatsapp + screenshots + unknown + mkv;
    let errors = count_log_errors(&final_output_dir.join("Technical Files/logs"));
    let csv_files = vec!["photos.csv", "videos.csv", "unknown_time.csv", "mkv_files.csv"];
    let log_files = vec!["media_cleaning.log", "metadata_extraction.log", "metadata_embedding.log", "sorting.log"];
    let metadata_fields: Vec<&str> = if let Some(meta) = metadata.first() {
        let mut fields = vec!["media_path", "json_path"];
        if meta.exif_date.is_some() { fields.push("exif_date"); }
        if meta.gps_latitude.is_some() { fields.push("gps_latitude"); }
        if meta.gps_longitude.is_some() { fields.push("gps_longitude"); }
        if meta.gps_altitude.is_some() { fields.push("gps_altitude"); }
        if meta.camera_make.is_some() { fields.push("camera_make"); }
        if meta.camera_model.is_some() { fields.push("camera_model"); }
        fields
    } else {
        vec!["media_path", "json_path", "exif_date", "gps_latitude", "gps_longitude", "gps_altitude", "camera_make", "camera_model"]
    };
    html_report::generate_html_report(
        &final_output_dir,
        total, photos, videos, whatsapp, screenshots, unknown, mkv, errors,
        &csv_files, &log_files, &metadata_fields,
    );

    // Print summary
    MetaSortUI::print_summary(
        photos, videos, whatsapp, screenshots, unknown, mkv, errors,
        &final_output_dir.to_string_lossy()
    );

    // Delete MetaSort_temp folder after all processing
    if temp_dir.exists() {
        match fs_extra::dir::remove(temp_dir) {
            Ok(_) => MetaSortUI::print_info(&format!("Deleted temporary folder: {}", temp_dir.display())),
            Err(e) => MetaSortUI::print_warning(&format!("Could not delete temporary folder: {} (Error: {})", temp_dir.display(), e)),
        }
    }

    MetaSortUI::print_footer();
}

fn count_files(dir: &PathBuf) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .count()
}

fn count_files_in_directory(path: &str) -> usize {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .count()
}

fn count_log_errors(logs_dir: &PathBuf) -> usize {
    let mut errors = 0;
    if let Ok(entries) = fs::read_dir(logs_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                errors += content.matches("[ERROR]").count();
                errors += content.matches("[FAILED]").count();
            }
        }
    }
    errors
} 