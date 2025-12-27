# Path Handling Guide

## Overview

MetaSort v1.1.0-bberka supports various path formats across Windows, macOS, and Linux platforms. This guide covers supported formats, edge cases, troubleshooting, and best practices.

## Supported Path Formats

### Windows

| Format | Example | Notes |
|--------|---------|-------|
| Simple path | `C:\Photos` | Standard Windows path |
| Path with spaces | `C:\My Documents\Photos` | Automatically handled, quotes optional |
| Quoted path (double) | `"C:\My Path\Photos"` | Quotes automatically removed |
| Quoted path (single) | `'C:\My Path\Photos'` | Single quotes also supported |
| UNC path | `\\server\share\folder` | Network UNC path |
| UNC with IP | `\\192.168.1.100\c$` | IP-based UNC, admin shares supported |
| Mapped drive | `Z:\Photos` | Network drive mapped to letter |
| External drive | `E:\Backup` | USB/external storage |

### macOS/Linux

| Format | Example | Notes |
|--------|---------|-------|
| Simple path | `/home/user/photos` | Standard Unix path |
| Path with spaces | `/Users/John Doe/Photos` | Automatically handled |
| Quoted path | `"/Users/My Path/Photos"` | Quotes automatically removed |
| Network mount | `/mnt/network/share` | Network filesystem mount |
| Home directory | `~/Photos` | Shell expansion required |

## Network Path Considerations

### Performance Warning

When MetaSort detects a network path (UNC or mapped drive), it displays this warning:

```
[WARNING] Network path detected!
Network storage may experience slower performance due to I/O latency.
RECOMMENDATION: Use 'Slow' concurrency mode (2-4 threads) to avoid overwhelming the network.
Processing large datasets over network can take significantly longer.
```

### Why Slow Concurrency for Network Paths?

**Technical Reasons:**
- Network I/O has higher latency than local disk (10-100ms vs <1ms)
- High concurrency can saturate network bandwidth
- Multiple simultaneous operations may cause timeouts
- Some network filesystems don't handle parallel I/O well

**Recommended Settings:**
- **Network input/output**: Use "Slow" mode (2-4 threads)
- **Local SSD**: Use "Medium" or "Unlimited" mode
- **Local HDD**: Use "Medium" mode

### Best Practices for Network Paths

1. **Prefer local output directory**
   - Use a local disk for the output directory even if input is on network
   - This reduces network write operations during processing
   ```bash
   # Good: Network input, local output
   metasort -i "\\server\photos" -o "C:\Local\Output"

   # Avoid: Both on network
   metasort -i "\\server\photos" -o "\\server\output"
   ```

2. **Map drives for stability**
   - Instead of `\\server\share`, map to a drive letter (e.g., `Z:\`)
   - Provides better Windows compatibility
   - More reliable across reboots
   ```cmd
   net use Z: \\server\share /persistent:yes
   metasort -i "Z:\photos" -o "C:\Output"
   ```

3. **Verify accessibility before starting**
   - Ensure you have proper credentials
   - Test network connectivity
   - Check available space on network location

4. **Use slow concurrency**
   - Always select "Slow" mode for network input/output
   - Balances speed and reliability
   - Prevents network timeouts

## Troubleshooting

### UNC Path Not Accessible

**Error:**
```
[ERROR] Invalid input directory: UNC path not accessible: \\server\share - Cannot access UNC path
```

**Solutions:**

1. **Verify path is accessible in Windows File Explorer**
   - Open File Explorer
   - Type `\\server\share` in the address bar
   - If it prompts for credentials, enter them

2. **Check network credentials in Windows Credential Manager**
   - Press `Win + R`, type `control /name Microsoft.CredentialManager`
   - Add Windows credentials for the server
   - Use format: `server` or `server.domain.com`

3. **Try mapping the drive first**
   ```cmd
   net use Z: \\server\share /user:DOMAIN\username password
   ```
   Then use `Z:\` instead of `\\server\share`

4. **Use IP address if DNS is failing**
   ```bash
   # Instead of
   \\server\share

   # Try
   \\192.168.1.100\share
   ```

### Permission Denied

**Error:**
```
[ERROR] Invalid input directory: Permission denied: C:\My Path
```

**Solutions:**

1. **Run MetaSort as Administrator**
   - Right-click on `metasort.exe`
   - Select "Run as administrator"

2. **Check folder permissions**
   - Right-click folder → Properties → Security
   - Ensure your user has Read & Execute permissions
   - For output folder, ensure Write permission

3. **Disable antivirus temporarily**
   - Some antivirus software blocks file access
   - Add MetaSort to antivirus exclusions

4. **Check if folder is locked**
   - Close any programs using the folder
   - Use Process Explorer to find locking processes

### Path Does Not Exist

**Error:**
```
[ERROR] Invalid input directory: Path does not exist: C:\NonExistent
```

**Solutions:**

1. **Double-check the path**
   - Verify spelling and capitalization
   - Ensure all parent directories exist

2. **Check drive letter assignment**
   - For external drives, verify drive letter
   - Use Disk Management to check

3. **For network paths, verify server is online**
   ```cmd
   ping server
   ping 192.168.1.100
   ```

4. **Use absolute paths, not relative**
   ```bash
   # Good
   C:\Users\John\Photos

   # Avoid
   ..\Photos
   ```

## Advanced Usage

### Paths with Special Characters

MetaSort handles paths with:

- **Spaces**: `C:\My Documents`
- **Apostrophes**: `C:\User's Photos`
- **Parentheses**: `C:\Photos (Backup)`
- **Brackets**: `C:\[Archive]\Photos`
- **Ampersands**: `C:\Photos & Videos`
- **Unicode**: `C:\照片` (Chinese), `C:\Фото` (Cyrillic)

### Escaped Quotes in Paths

For paths containing quote characters in folder names:

**Windows CMD:**
```cmd
metasort --input "C:\My \"Photos\"" --output "D:\Output"
```

**PowerShell:**
```powershell
metasort --input 'C:\My "Photos"' --output 'D:\Output'
```

**Bash (WSL/Linux):**
```bash
metasort --input "C:\\My \"Photos\"" --output "D:\\Output"
```

### Drag-and-Drop in Interactive Mode

**Windows:**
1. Run `metasort.exe`
2. When prompted for input path, drag folder from File Explorer
3. Windows automatically adds quotes if path contains spaces
4. MetaSort removes quotes during validation

**macOS/Linux:**
- Drag-and-drop works in most terminals
- Terminal automatically escapes spaces

### Working with Symbolic Links

**Windows (Junctions/Symlinks):**
- MetaSort follows symbolic links
- Processes actual target location
- Ensure target is accessible

**Unix (Symlinks):**
- Automatically dereferenced
- Original path used in reports

## Performance Guidelines

### Expected Performance by Storage Type

| Storage Type | Concurrency | Expected Speedup | Notes |
|--------------|-------------|------------------|-------|
| Local SSD | Medium/Unlimited | 3-5x | Near theoretical maximum |
| Local HDD | Medium | 2-3x | I/O limited |
| USB 3.0 SSD | Medium | 2-4x | USB overhead |
| USB 2.0 HDD | Slow | 1.5-2x | Bandwidth limited |
| Gigabit Network | Slow | 1.5-2.5x | Network latency |
| 100Mb Network | Slow | 1-1.5x | Severely limited |

### Optimization Tips

1. **Use local storage for output**
   - Dramatically improves performance
   - Reduces network traffic

2. **Close unnecessary applications**
   - Frees up CPU and memory
   - Reduces disk contention

3. **Disable real-time antivirus scanning**
   - Temporarily for MetaSort directories
   - Significantly faster file operations

4. **Ensure sufficient free space**
   - Requires 3x input folder size
   - SSD performance degrades when full

## Implementation Details

### Path Normalization

MetaSort normalizes paths by:

1. Trimming leading/trailing whitespace
2. Removing outer quotes (preserving inner quotes)
3. Handling escaped quotes (`\"` and `\'`)
4. Converting to `PathBuf` for consistent handling

### Path Type Detection

**On Windows:**
- **UNC paths**: Detected by `\\` or `//` prefix
- **Network mapped drives**: Using Windows API `GetDriveTypeW`
- **Local disks**: All other drive letters

**On Unix:**
- **Network mounts**: Paths under `/mnt/`, `/net/`
- **Local storage**: All other paths

### Canonicalization Strategy

MetaSort uses `fs::canonicalize()` selectively:

- **Used for**: Local paths, relative paths, symlinks
- **Avoided for**: UNC paths on Windows (can cause errors)
- **Fallback**: Returns original path if canonicalization fails

**Example:**
```rust
// Safe for UNC paths
let safe_path = path_utils::safe_canonicalize(path)
    .unwrap_or_else(|_| path.to_path_buf());
```

## CLI vs Interactive Mode

### When to Use CLI Mode

- **Automation**: Scripts, batch processing
- **Consistency**: Same parameters every time
- **CI/CD**: Automated testing/deployment
- **Unattended**: No user interaction needed

**Example automation script (Windows):**
```bat
@echo off
set INPUT=\\nas\photos\takeout
set OUTPUT=D:\Organized
metasort -i "%INPUT%" -o "%OUTPUT%" -c slow --yes
if %ERRORLEVEL% EQU 0 (
    echo Success!
) else (
    echo Failed!
)
```

### When to Use Interactive Mode

- **First time users**: Guided prompts
- **One-off processing**: Not repeated
- **Exploring options**: Learn available settings
- **Complex paths**: Drag-and-drop convenience

## Common Patterns

### Pattern 1: Network Input → Local Output

**Best for**: Processing network backups, accessing NAS

```bash
# CLI mode
metasort -i "\\nas\photos" -o "C:\Local\Output" -c slow --yes

# Interactive mode
# 1. Run: metasort
# 2. Input: \\nas\photos
# 3. Output: C:\Local\Output
# 4. Concurrency: Choose "Slow"
```

### Pattern 2: External Drive Processing

**Best for**: Processing photos from external USB drive

```bash
# CLI mode
metasort -i "E:\DCIM" -o "D:\Organized" -c medium

# Wait for prompt before disconnecting drive
```

### Pattern 3: Batch Processing Multiple Takeouts

**Best for**: Processing multiple Google Takeout archives

```bash
#!/bin/bash
for dir in /mnt/storage/takeout-*; do
    output="/mnt/storage/processed/$(basename "$dir")"
    metasort -i "$dir" -o "$output" -c medium --yes
done
```

## Error Recovery

### What to Do If Processing Fails

1. **Check logs** in `MetaSort_Output/Technical Files/logs/`
   - `media_cleaning.log`
   - `metadata_extraction.log`
   - `metadata_embedding.log`
   - `sorting.log`

2. **Look for error patterns**
   - Permission errors → Run as admin
   - Network errors → Check connectivity
   - Space errors → Free up disk space

3. **Resume from where it failed**
   - MetaSort doesn't support resume yet
   - But you can process failed files separately

4. **Report bugs**
   - GitHub issues: https://github.com/bberka/MetaSort/issues
   - Include logs and error messages

## Frequently Asked Questions

### Q: Can I use both input and output on network paths?

**A:** Yes, but it's not recommended. Performance will be significantly slower. Use local storage for output if possible.

### Q: Does MetaSort work with OneDrive/Dropbox paths?

**A:** Yes, but:
- Disable sync during processing
- Use local folders, not cloud-only files
- Performance may be slower

### Q: What about Windows path length limits (260 characters)?

**A:** MetaSort is subject to Windows MAX_PATH limit unless:
- Windows 10 1607+ with long path support enabled
- Or use UNC prefix: `\\?\C:\very\long\path`

To enable long paths:
```cmd
# Run as Administrator
reg add HKLM\SYSTEM\CurrentControlSet\Control\FileSystem /v LongPathsEnabled /t REG_DWORD /d 1
```

### Q: Can I process paths with non-English characters?

**A:** Yes, MetaSort handles Unicode paths correctly:
- Chinese: `C:\照片`
- Japanese: `C:\写真`
- Cyrillic: `C:\Фото`
- Arabic: `C:\الصور`

### Q: Does MetaSort modify the original files?

**A:** No! MetaSort:
1. Copies input folder to `MetaSort_temp`
2. Processes the copy
3. Outputs to `MetaSort_Output`
4. Leaves original untouched

## See Also

- [README.md](../README.md) - Main documentation
- [TESTING_CHECKLIST.md](TESTING_CHECKLIST.md) - Manual testing guide
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Codebase architecture
