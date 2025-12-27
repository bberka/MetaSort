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

1. **Rayon-Powered Concurrency**: Rewritten processing loop using a work-stealing thread pool. Parallelizes EXIF writing and I/O operations, maximizing CPU utilization.
2. **Standardized Logging**: Removed all emoji-based output and marketing fluff. Terminal output is now clean and optimized for logging/piping.
3. **Batch I/O**: Switched from sequential movement to batched file operations to reduce filesystem overhead.

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

```bash
# Basic usage
./target/release/metasort --input "/path/to/source" --output "/path/to/destination"

# Help and advanced flags
./target/release/metasort --help
```

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
