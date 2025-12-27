# MetaSort Installation Guide

This guide covers the setup for the high-performance fork (v1.1.0-bberka). Follow the steps for your specific operating system.

---

## macOS

### 1. Install System Dependencies

Open **Terminal** (found in Applications > Utilities) and execute these commands sequentially:

**Install Homebrew (Package Manager):**

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Install Rust (Compiler):**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

_Note: Press `1` when prompted. Restart your Terminal after this step._

**Install ExifTool:**

```bash
brew install exiftool
```

### 2. Build the Application

Navigate to your MetaSort folder in Terminal and run:

```bash
chmod +x scripts/build_macos.sh
./scripts/build_macos.sh
```

### 3. Execution

- Use `Run_MetaSort.command` to launch in a new window.
- Use `MetaSort.command` to launch in the existing window.

---

## Windows

### 1. Automated Setup (Recommended)

1. Locate the `scripts` folder inside the MetaSort directory.
2. Right-click `install_windows.bat` and select **Run as Administrator**.
3. Follow the on-screen prompts.

### 2. Manual Setup (Fallback)

If the automated script fails, perform the following:

1. **Install Rust**: Download and run the installer from [https://rustup.rs/](https://rustup.rs/).
2. **Install ExifTool**: Open Command Prompt and run:
   ```cmd
   winget install ExifTool.ExifTool
   ```
3. **Build**: In the MetaSort folder, run:
   ```cmd
   cargo build --release
   ```
4. **Run**:
   ```cmd
   ./target/release/metasort.exe
   ```

---

## Troubleshooting

### Dependency Errors

- **ExifTool not found**: Ensure the installation command finished without errors. On Windows, you may need to restart your terminal for the PATH to update.
- **Rust/Cargo not found**: Ensure you restarted your terminal after installing Rust.

### Permissions (macOS)

If you see a "Permission Denied" error when running scripts, run:

```bash
chmod +x scripts/*.sh
```

### macOS Security

If the application is blocked from running:

1. Open **System Settings** > **Privacy & Security**.
2. Scroll down and click **Open Anyway** next to the MetaSort notice.

---

## Technical Support

For detailed command-line options and performance configuration, refer to the main `README.md`.
