// path_utils.rs
// Path handling utilities for MetaSort
// Copyright (c) 2025 - Berkay (path handling enhancements for v1.1.0-bberka fork)
// Copyright (c) 2025 - Sanmith S. (original MetaSort work)
//
// Licensed under the Apache License, Version 2.0

use std::path::{Path, PathBuf};
use std::io;
use std::fs;

/// Represents the type of path for optimization and warning purposes
#[derive(Debug, PartialEq)]
pub enum PathType {
    LocalDisk,          // C:\, D:\, /mnt/disk
    NetworkUNC,         // \\server\share
    NetworkMapped,      // Z:\ (mapped to network)
    Unknown,
}

/// Result type for path operations
pub type PathResult<T> = Result<T, PathError>;

/// Path validation and processing errors
#[derive(Debug)]
pub enum PathError {
    DoesNotExist(String),
    NotAccessible(String),
    InvalidFormat(String),
    PermissionDenied(String),
    UNCNotSupported(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PathError::DoesNotExist(p) => write!(f, "Path does not exist: {}", p),
            PathError::NotAccessible(p) => write!(f, "Path is not accessible: {}", p),
            PathError::InvalidFormat(p) => write!(f, "Invalid path format: {}", p),
            PathError::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            PathError::UNCNotSupported(p) => write!(f, "UNC path not accessible: {}", p),
        }
    }
}

impl std::error::Error for PathError {}

/// Normalize path input: remove quotes, trim whitespace, handle escaped quotes
pub fn normalize_path_input(input: &str) -> String {
    let trimmed = input.trim();

    // Handle quoted paths (both single and double quotes)
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
       (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        // Remove outer quotes
        let unquoted = &trimmed[1..trimmed.len()-1];
        // Handle escaped quotes inside (e.g., "C:\My \"Photos\"")
        unquoted.replace("\\\"", "\"").replace("\\'", "'")
    } else {
        trimmed.to_string()
    }
}

/// Detect the type of path (local, UNC, mapped network drive)
#[cfg(target_os = "windows")]
pub fn detect_path_type(path: &Path) -> PathType {
    let path_str = path.to_string_lossy();

    // Check for UNC path
    if path_str.starts_with("\\\\") || path_str.starts_with("//") {
        return PathType::NetworkUNC;
    }

    // Check for mapped drive (requires Windows API call)
    if let Some(drive) = path_str.chars().next() {
        if drive.is_ascii_alphabetic() && path_str.len() > 1 && path_str.chars().nth(1) == Some(':') {
            // Use GetDriveType to determine if it's a network drive
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;

            let drive_root = format!("{}:\\", drive);
            let wide: Vec<u16> = OsStr::new(&drive_root)
                .encode_wide()
                .chain(Some(0))
                .collect();

            unsafe {
                let drive_type = winapi::um::fileapi::GetDriveTypeW(wide.as_ptr());
                // DRIVE_REMOTE = 4
                if drive_type == 4 {
                    return PathType::NetworkMapped;
                }
            }

            return PathType::LocalDisk;
        }
    }

    PathType::Unknown
}

#[cfg(not(target_os = "windows"))]
pub fn detect_path_type(path: &Path) -> PathType {
    // On Unix, check if path is under /mnt, /net, or other network mount points
    let path_str = path.to_string_lossy();

    if path_str.starts_with("/mnt/") || path_str.starts_with("/net/") {
        PathType::NetworkMapped
    } else {
        PathType::LocalDisk
    }
}

/// Validate input directory: must exist and be readable
pub fn validate_input_dir(path_str: &str) -> PathResult<PathBuf> {
    let normalized = normalize_path_input(path_str);
    let path = PathBuf::from(&normalized);

    // Check existence
    if !path.exists() {
        return Err(PathError::DoesNotExist(normalized));
    }

    // Check if it's a directory
    if !path.is_dir() {
        return Err(PathError::InvalidFormat(format!("{} is not a directory", normalized)));
    }

    // Try to read the directory to verify accessibility
    match fs::read_dir(&path) {
        Ok(_) => {},
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(PathError::PermissionDenied(normalized));
        },
        Err(e) => {
            return Err(PathError::NotAccessible(format!("{}: {}", normalized, e)));
        }
    }

    // For UNC paths on Windows, verify accessibility explicitly
    #[cfg(target_os = "windows")]
    if detect_path_type(&path) == PathType::NetworkUNC {
        // Try to access a file in the directory to verify network connectivity
        match fs::read_dir(&path) {
            Ok(mut entries) => {
                // Try to read at least one entry to verify it's truly accessible
                if let Some(Err(_)) = entries.next() {
                    return Err(PathError::UNCNotSupported(
                        format!("{} - Network path is not accessible or requires authentication", normalized)
                    ));
                }
            },
            Err(_) => {
                return Err(PathError::UNCNotSupported(
                    format!("{} - Cannot access UNC path", normalized)
                ));
            }
        }
    }

    Ok(path)
}

/// Validate output directory: create if doesn't exist (with user confirmation)
pub fn validate_output_dir(path_str: &str, auto_create: bool) -> PathResult<PathBuf> {
    let normalized = normalize_path_input(path_str);
    let path = PathBuf::from(&normalized);

    if path.exists() {
        // Verify it's a directory and writable
        if !path.is_dir() {
            return Err(PathError::InvalidFormat(format!("{} exists but is not a directory", normalized)));
        }

        // Test write access by trying to create a temp file
        let test_file = path.join(".metasort_test_write");
        match fs::File::create(&test_file) {
            Ok(_) => {
                let _ = fs::remove_file(&test_file); // Clean up
                Ok(path)
            },
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                Err(PathError::PermissionDenied(normalized))
            },
            Err(e) => {
                Err(PathError::NotAccessible(format!("{}: {}", normalized, e)))
            }
        }
    } else {
        // Directory doesn't exist
        if auto_create {
            // Create directory
            match fs::create_dir_all(&path) {
                Ok(_) => Ok(path),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    Err(PathError::PermissionDenied(normalized))
                },
                Err(e) => {
                    Err(PathError::NotAccessible(format!("Cannot create {}: {}", normalized, e)))
                }
            }
        } else {
            Err(PathError::DoesNotExist(normalized))
        }
    }
}

/// Get warning message for network paths
pub fn get_network_path_warning(path_type: PathType) -> Option<String> {
    match path_type {
        PathType::NetworkUNC | PathType::NetworkMapped => {
            Some(format!(
                "\n[WARNING] Network path detected!\n\
                 Network storage may experience slower performance due to I/O latency.\n\
                 RECOMMENDATION: Use 'Slow' concurrency mode (2-4 threads) to avoid overwhelming the network.\n\
                 Processing large datasets over network can take significantly longer.\n"
            ))
        },
        _ => None,
    }
}

/// Check if path should be canonicalized (avoid for UNC paths on Windows)
pub fn should_canonicalize(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        detect_path_type(path) != PathType::NetworkUNC
    }

    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

/// Safe canonicalization that handles UNC paths properly
pub fn safe_canonicalize(path: &Path) -> io::Result<PathBuf> {
    if should_canonicalize(path) {
        fs::canonicalize(path)
    } else {
        // For UNC paths, just return the absolute path without canonicalizing
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_input_double_quotes() {
        assert_eq!(normalize_path_input("\"C:\\My Path\""), "C:\\My Path");
    }

    #[test]
    fn test_normalize_path_input_single_quotes() {
        assert_eq!(normalize_path_input("'C:\\My Path'"), "C:\\My Path");
    }

    #[test]
    fn test_normalize_path_input_no_quotes() {
        assert_eq!(normalize_path_input("C:\\My Path"), "C:\\My Path");
    }

    #[test]
    fn test_normalize_path_input_escaped_quotes() {
        assert_eq!(normalize_path_input("\"C:\\My \\\"Photos\\\"\""), "C:\\My \"Photos\"");
    }

    #[test]
    fn test_normalize_path_input_whitespace() {
        assert_eq!(normalize_path_input("  \"C:\\Path\"  "), "C:\\Path");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_detect_path_type_unc() {
        let path = Path::new("\\\\server\\share\\folder");
        assert_eq!(detect_path_type(path), PathType::NetworkUNC);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_detect_path_type_unc_forward_slash() {
        let path = Path::new("//server/share/folder");
        assert_eq!(detect_path_type(path), PathType::NetworkUNC);
    }

    #[test]
    fn test_validate_input_dir_not_exists() {
        let result = validate_input_dir("Z:\\NonExistent\\Path\\12345");
        assert!(result.is_err());
        if let Err(PathError::DoesNotExist(_)) = result {
            // Expected
        } else {
            panic!("Expected DoesNotExist error");
        }
    }

    #[test]
    fn test_get_network_path_warning_local() {
        let warning = get_network_path_warning(PathType::LocalDisk);
        assert!(warning.is_none());
    }

    #[test]
    fn test_get_network_path_warning_unc() {
        let warning = get_network_path_warning(PathType::NetworkUNC);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Network path detected"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_should_canonicalize_local() {
        let path = Path::new("C:\\Users\\Test");
        assert!(should_canonicalize(path));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_should_canonicalize_unc() {
        let path = Path::new("\\\\server\\share");
        assert!(!should_canonicalize(path));
    }
}
