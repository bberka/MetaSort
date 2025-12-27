# MetaSort v1.1.0-bberka

MetaSort is a high-performance Rust utility for processing and organizing Google Photos Takeout exports. It correlates detached JSON metadata back to media files and embeds information directly into EXIF headers while sorting files into a logical hierarchy.

---

## Technical Workflow

- **Sanitization**: Standardizes filenames and removes duplicate Google suffixes.
- **Correlation**: Maps `.json` sidecar files to media based on filename heuristics.
- **Embedding**: Writes DateTaken, GPS, and Camera Model into EXIF metadata.
- **Sorting**: Organizes library into `Year/Month/Type` (Photos/Videos/Screenshots).
- **Recovery**: Multi-stage date fallback: JSON -> Filename -> EXIF -> File System.

---

## Custom Fork Enhancements

1. **Rayon-Powered Concurrency**: Comprehensive parallelization across all major operations (metadata extraction, file sorting, and media cleaning). User-selectable concurrency levels maximize performance while respecting system resources.
2. **Standardized Logging**: Removed all emoji-based output and marketing fluff. Terminal output is now clean and optimized for logging/piping.
3. **Batch I/O**: Switched from sequential movement to batched file operations to reduce filesystem overhead.
4. **Performance**: 3-5x faster on typical datasets (10k+ files) compared to sequential processing.
5. **Comprehensive Path Handling**: Full support for paths with spaces, quoted paths, UNC network paths (`\\server\share`), and mapped network drives. Automatic quote removal and path normalization.
6. **CLI Mode**: Optional command-line arguments (`--input`, `--output`, `--concurrency`) alongside the existing interactive mode for scripting and automation.
7. **Enhanced Error Handling**: Graceful failure recovery with actionable error messages and troubleshooting guidance, especially for network paths and permission issues.

---

## Installation

### Prerequisites

- **ExifTool**: Required for metadata manipulation.
- **Rust Toolchain**: Required for compilation.

### Build Instructions

```bash
# macOS/Linux
brew install exiftool
cargo build --release

# Windows (PowerShell)
winget install ExifTool.ExifTool
cargo build --release
```

---

## Usage

MetaSort supports both interactive mode (guided prompts) and CLI mode (command-line arguments).

### Interactive Mode (Recommended for first-time users)

```bash
./target/release/metasort
```

Follow the on-screen prompts to:
1. Specify input directory (drag-and-drop supported)
2. Specify output directory
3. Select concurrency level
4. Choose whether to separate WhatsApp/Screenshots

### CLI Mode (For automation and scripting)

```bash
# Basic usage
./target/release/metasort --input "/path/to/source" --output "/path/to/destination"

# With all options
./target/release/metasort \
  --input "C:\My Photos\Takeout" \
  --output "D:\Organized Photos" \
  --concurrency medium \
  --separate-whatsapp-screenshots \
  --yes

# View all options
./target/release/metasort --help
```

### Supported Path Types

MetaSort handles various path formats on Windows:

- **Paths with spaces**: `"C:\My Documents\Photos"` or `C:\My Documents\Photos`
- **UNC network paths**: `\\server\share\folder` or `\\192.168.1.100\c$\Photos`
- **Mapped network drives**: `Z:\Photos` (where Z: is mapped to network location)
- **External drives**: `E:\Backup\Photos`
- **Quoted paths**: Both single (`'...'`) and double (`"..."`) quotes supported

**Note:** Network paths (UNC or mapped drives) may experience slower performance. MetaSort will display a warning and recommend using "Slow" concurrency mode.

### Command-Line Options

```
Options:
  -i, --input <INPUT_DIR>              Input directory (Google Photos Takeout folder)
  -o, --output <OUTPUT_DIR>            Output directory (where MetaSort creates working folders)
  -c, --concurrency <LEVEL>            Concurrency level: slow, medium, unlimited [default: medium]
      --separate-whatsapp-screenshots  Separate WhatsApp and Screenshot images
  -y, --yes                            Skip confirmation prompts
  -h, --help                           Print help
  -V, --version                        Print version
```

### Examples

```bash
# Process local folder with medium concurrency
metasort -i "C:\Takeout" -o "D:\Output" -c medium

# Process network share with slow concurrency (recommended)
metasort -i "\\server\photos" -o "C:\Local Output" -c slow --yes

# Interactive mode with all prompts
metasort
```

---

## Concurrency Configuration

MetaSort allows you to select the concurrency level at startup to balance performance and resource usage:

### Concurrency Levels

1. **Slow (2-4 threads)**
   - Best for: Systems with limited resources, older hardware, or when running other heavy applications
   - Thread count: 2-4 threads (capped at 4, minimum 2)
   - Resource impact: Minimal CPU and memory usage

2. **Medium (half CPU cores) - RECOMMENDED**
   - Best for: Most users, balanced performance and system responsiveness
   - Thread count: Half of available CPU cores (minimum 2)
   - Resource impact: Moderate CPU usage, leaves headroom for other tasks
   - Performance: 2-4x faster than sequential processing

3. **Unlimited (all CPU cores)**
   - Best for: Maximum speed on dedicated processing, large datasets (50k+ files)
   - Thread count: All available CPU cores
   - Resource impact: High CPU usage, may affect system responsiveness
   - Performance: 3-5x faster than sequential processing

### Performance Guidelines

**Hardware Recommendations:**

- **2-4 core systems**: Use Slow or Medium
- **4-8 core systems**: Use Medium (recommended for most users)
- **8+ core systems**: Use Medium or Unlimited

**Storage Considerations:**

- **SSD storage**: Near theoretical speedup (3-5x with Medium/Unlimited)
- **HDD storage**: 50-70% of theoretical speedup (I/O limited)
- **Network drives**: Use Slow to avoid overwhelming the network

**Dataset Size:**

- **Small (<1,000 files)**: Lower speedup due to parallelization overhead
- **Medium (1,000-10,000 files)**: 2-3x speedup with Medium
- **Large (10,000+ files)**: 3-5x speedup with Medium/Unlimited

---

## Output Structure

```text
MetaSort_Output/
├── Media Files/
│   ├── 2023/
│   │   ├── 01_January/
│   │   │   ├── Photos/
│   │   │   └── Videos/
└── Technical Files/
    ├── report.html          # Web-based processing summary
    ├── processing_log.csv   # Machine-readable action log
    └── error_log.txt        # Failed operations and reasonings
```

---

## Support & Original Project

This is a modified fork of the original MetaSort project.

- **Original Repository**: [https://github.com/iamsanmith/MetaSort](https://github.com/iamsanmith/MetaSort)
- **Support Original Author**: [PayPal](https://www.paypal.com/paypalme/iamsanmith) | [Ko-fi](https://ko-fi.com/L3L01JGIUW)

---

## License

Copyright (c) 2025 - Sanmith S.
Portions Copyright (c) 2025 - Berkay.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at:
http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
