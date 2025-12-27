// platform.rs
// Cross-platform utilities for MetaSort

use std::process::Command;
use std::env;

#[cfg(target_os = "windows")]
pub const EXIFTOOL_CMD: &str = "exiftool.exe";

#[cfg(not(target_os = "windows"))]
pub const EXIFTOOL_CMD: &str = "exiftool";

/// Check if exiftool is available on the system
pub fn is_exiftool_available() -> bool {
    let output = Command::new(EXIFTOOL_CMD)
        .arg("-ver")
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Get the exiftool command with proper path handling
pub fn get_exiftool_command() -> Command {
    Command::new(EXIFTOOL_CMD)
}

/// Get platform-specific installation instructions
pub fn get_installation_instructions() -> String {
    match env::consts::OS {
        "windows" => {
            r#"
Windows Installation Instructions:

1. Install Rust: https://rustup.rs/
2. Install exiftool using one of these methods:
   
   Option A (Recommended): Using Chocolatey
   ```cmd
   choco install exiftool
   ```
   
   Option B: Using winget
   ```cmd
   winget install ExifTool.ExifTool
   ```
   
   Option C: Manual installation
   - Download from https://exiftool.org/
   - Extract to C:\exiftool
   - Add C:\exiftool to your PATH environment variable
   
3. Restart your command prompt after installation
"#.to_string()
        },
        "macos" => {
            r#"
macOS Installation Instructions:

1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Install exiftool:
   ```sh
   brew install exiftool
   ```
"#.to_string()
        },
        _ => {
            r#"
Linux Installation Instructions:

1. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2. Install exiftool:
   ```sh
   # Ubuntu/Debian
   sudo apt-get install exiftool
   
   # CentOS/RHEL/Fedora
   sudo yum install perl-Image-ExifTool
   # or
   sudo dnf install perl-Image-ExifTool
   ```
"#.to_string()
        }
    }
} 