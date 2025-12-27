# Path Handling Testing Checklist

This checklist covers manual testing scenarios for the enhanced path handling features in MetaSort v1.1.0-bberka.

## Local Paths - Windows

- [ ] Simple path: `C:\Photos`
- [ ] Path with spaces: `C:\My Documents\Photos`
- [ ] Quoted path (double): `"C:\My Path\Photos"`
- [ ] Quoted path (single): `'C:\My Path\Photos'`
- [ ] Path with apostrophe: `C:\User's Photos`
- [ ] Deep nested: `C:\Very\Long\Path\With\Many\Folders\Photos`
- [ ] Root drive: `D:\`
- [ ] External USB drive: `E:\Backup`
- [ ] Path with parentheses: `C:\Photos (2024)`
- [ ] Path with brackets: `C:\[Archive]\Photos`

## Local Paths - Unix/macOS

- [ ] Simple path: `/home/user/photos`
- [ ] Path with spaces: `/Users/John Doe/Photos`
- [ ] Quoted path: `"/Users/My Path/Photos"`
- [ ] Path with apostrophe: `/Users/user's/Photos`
- [ ] Home directory: `~/Photos`
- [ ] Symlinked path

## Network Paths - Windows

- [ ] UNC server: `\\server\share\photos`
- [ ] UNC with IP: `\\192.168.1.100\share`
- [ ] UNC admin share: `\\server\c$\Photos`
- [ ] UNC with spaces: `\\server\My Share\Photos`
- [ ] Quoted UNC path: `"\\server\share\photos"`
- [ ] Mapped drive: `Z:\Photos` (where Z: is mapped to network)
- [ ] DFS path: `\\domain\dfs\share\photos`

## CLI Mode

### Basic Usage
- [ ] `metasort --input "C:\My Path" --output "D:\Output"`
- [ ] `metasort -i C:\Photos -o D:\Output`
- [ ] Help command: `metasort --help`
- [ ] Version: `metasort --version`

### Concurrency Options
- [ ] `--concurrency slow`
- [ ] `--concurrency medium`
- [ ] `--concurrency unlimited`
- [ ] `-c 1` (slow)
- [ ] `-c 2` (medium)
- [ ] `-c 3` (unlimited)
- [ ] Invalid concurrency (should default to medium with warning)

### Flags
- [ ] `--separate-whatsapp-screenshots`
- [ ] `--yes` (auto-confirm)
- [ ] Combined: `--input C:\Photos --output D:\Out --concurrency slow --separate-whatsapp-screenshots --yes`

### Network Paths in CLI
- [ ] `--input "\\server\share" --output "C:\Local Output"`
- [ ] Verify network warning is displayed
- [ ] UNC input with local output
- [ ] Local input with UNC output (not recommended but should work)

## Interactive Mode

### Path Input
- [ ] Drag and drop path with spaces (Windows)
- [ ] Type quoted path manually
- [ ] Type UNC path manually
- [ ] Paste path from File Explorer
- [ ] Path with trailing backslash

### User Experience
- [ ] Network path warning displayed correctly
- [ ] Output dir creation prompt works
- [ ] Confirmation prompts function
- [ ] Help text is clear and helpful

### Concurrency Selection
- [ ] Interactive concurrency prompt works
- [ ] Network path warning recommends "Slow" mode
- [ ] Thread count displayed correctly

## Error Handling

### Input Directory Errors
- [ ] Non-existent path: Clear error message displayed
- [ ] Permission denied: Helpful troubleshooting steps shown
- [ ] Inaccessible UNC path: Network troubleshooting guidance provided
- [ ] Invalid path format: Descriptive error
- [ ] File instead of directory: Appropriate error

### Output Directory Errors
- [ ] Output path exists as file (not directory): Error shown
- [ ] Cannot create output directory: Permission error
- [ ] Read-only output location: Clear error

### Copy Operation Errors
- [ ] Insufficient disk space: Helpful error with suggestions
- [ ] Permission denied during copy: Administrator suggestion
- [ ] Network failure mid-copy: Network troubleshooting
- [ ] File in use: Clear error message

## Performance Testing

### Local Storage
- [ ] SSD: Verify expected speedup (3-5x with medium/unlimited)
- [ ] HDD: Verify reasonable performance (2-3x with medium)
- [ ] Small dataset (<1,000 files): Works correctly
- [ ] Medium dataset (1,000-10,000 files): Good performance
- [ ] Large dataset (10,000+ files): Stable processing

### Network Storage
- [ ] UNC path with Slow mode: Stable and no timeouts
- [ ] UNC path with Unlimited mode: Check for issues (not recommended)
- [ ] Mapped network drive: Same behavior as UNC
- [ ] Network disconnection during processing: Graceful error

## Cross-Platform Testing

### Windows 10/11
- [ ] All path types work correctly
- [ ] UNC path detection functions
- [ ] Network drive detection accurate
- [ ] CLI and interactive modes both work

### macOS
- [ ] Standard paths work
- [ ] Network mounts detected
- [ ] Spaces in paths handled
- [ ] CLI and interactive modes both work

### Linux
- [ ] Standard paths work
- [ ] Network mounts (NFS, CIFS) detected
- [ ] Symlinks handled correctly
- [ ] CLI and interactive modes both work

## Regression Testing

### Existing Functionality
- [ ] JSON cleaning works as before
- [ ] Metadata extraction unchanged
- [ ] EXIF embedding functions correctly
- [ ] File sorting maintains structure
- [ ] HTML report generated successfully
- [ ] CSV reports created
- [ ] Logs written correctly

### Backward Compatibility
- [ ] Simple paths without quotes still work
- [ ] Existing scripts/automation unaffected
- [ ] Output structure unchanged

## Edge Cases

### Unusual Paths
- [ ] Path exactly 260 characters (Windows MAX_PATH limit)
- [ ] Path with unicode characters: `C:\照片`
- [ ] Path with mixed separators: `C:\Users/name/folder` (Windows)
- [ ] Path with multiple spaces: `C:\My   Photos`
- [ ] Path ending with space (should be trimmed)

### Quoted Path Variations
- [ ] Nested quotes: `"C:\My \"Photos\""`
- [ ] Single quotes inside double: `"C:\User's Photos"`
- [ ] Empty quotes: `""` (should error)
- [ ] Unmatched quotes: `"C:\Photos` (handled gracefully)

### Special Scenarios
- [ ] Running from network location
- [ ] Temp directory on different drive than output
- [ ] Very large input directory (100+ GB)
- [ ] Input directory with many subdirectories (deep nesting)

## Documentation Verification

- [ ] README.md accurately describes CLI usage
- [ ] PATH_HANDLING.md examples all work
- [ ] Error messages match documentation
- [ ] Help text (`--help`) is complete and accurate

## License Compliance

- [ ] All modified files have appropriate notices
- [ ] README credits original author
- [ ] LICENSE file unchanged
- [ ] Documentation mentions fork relationship

---

## Testing Notes

**Date:** ___________

**Tester:** ___________

**Platform:** ___________

**Build Version:** ___________

### Issues Found:

1.
2.
3.

### Additional Comments:


