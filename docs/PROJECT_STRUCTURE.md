### PROJECT_STRUCTURE.md

# Project Architecture and Directory Layout

## Root Directory

- `Run_MetaSort.command`: macOS entry point (new terminal instance).
- `MetaSort.command`: macOS entry point (current terminal instance).
- `Cargo.toml`: Rust package manifest and dependency definitions.
- `README.md`: Primary documentation.
- `LICENSE.txt`: Apache 2.0 license text.

## Directory Breakdown

### /src/ (Core Logic)

- `main.rs`: Application entry point and CLI argument parsing.
- `platform.rs`: Abstraction layer for cross-platform filesystem operations.
- `ui.rs`: Console UI implementation and progress bar management.
- `media_cleaning.rs`: Logic for filename sanitization and duplicate handling.
- `metadata_extraction.rs`: Parser for Google Takeout JSON sidecar files.
- `metadata_embed.rs`: Wrapper for ExifTool interaction and metadata writing.
- `sort_to_folders.rs`: Directory hierarchy generation logic.
- `csv_report.rs`: Machine-readable processing log generator.
- `html_report.rs`: Statistical summary and web-report generator.
- `filename_date_guess.rs`: Regex-based date extraction from filename patterns.
- `utils.rs`: Shared helper functions and common types.

### /scripts/ (Automation)

- `build_macos.sh`: Compilation and launcher generation for Darwin targets.
- `build_windows.bat`: MSVC/MinGW build automation.
- `build_all.sh`: Shell-agnostic build dispatcher.
- `install_windows.ps1`: Automated dependency and environment setup (PowerShell).
- `install_windows.bat`: Legacy batch wrapper for environment setup.

### /docs/ (Technical Reference)

- `SIMPLE_INSTALL.md`: Dependency setup guide.
- `CROSS_PLATFORM_CHANGES.md`: Internal documentation regarding platform-specific implementation details.

### /assets/ (Static Resources)

- `logo.png`: Application branding.
- `upi.png`: Payment gateway assets.

---

## Dependency Graph (High Level)

1. **Input**: CLI Arguments -> `main.rs`
2. **Indexing**: `media_cleaning.rs` scans source directory.
3. **Metadata**: `metadata_extraction.rs` + `filename_date_guess.rs` correlate data.
4. **Execution**: `metadata_embed.rs` invokes ExifTool via `rayon` (concurrency).
5. **Organization**: `sort_to_folders.rs` moves/copies files to final hierarchy.
6. **Reporting**: `csv_report.rs` + `html_report.rs` finalize logs.
