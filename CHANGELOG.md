# Changelog

All notable changes to FlashKraft will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Real Device Writing**: Actual OS image writing to block devices
  - Uses `block-utils` crate for professional device handling
  - Proper device unmounting before writing
  - Block device erasure for clean writes
  - 4MB block size for efficient writing
  - Comprehensive error handling and permission checks
  - Requires root/sudo privileges for writing
  - Works with USB drives, SD cards, and other block devices
  - Automatic sync after writing completes

### Fixed
- **Code Cleanup**: Removed all compiler warnings
  - Fixed unused `Command` warnings in test functions (added `let _ =`)
  - Fixed useless comparison warning in drive detection test
  - All tests now pass without warnings
  - Clean clippy output with zero warnings
  - Code is fully warning-free (except external dependency warnings)
- **Drive Detection**: Comprehensive block device detection improvements
  - Now detects ALL block devices, not just mounted filesystems
  - Reads from `/sys/block` to find all physical and virtual devices
  - Shows devices even if not mounted (e.g., USB drives, SD cards)
  - Detects device model names from sysfs (`/sys/block/*/device/model`)
  - Detects vendor information for better device identification
  - Filters out virtual devices (loop, ram, dm-*)
  - Correctly maps partitions to parent devices

### Added
- **Bootstrap Icons Integration**: Professional icon set throughout the application
  - File icon for image selection
  - Device/SSD icon for target selection
  - Lightning bolt icon for flash operation
  - Check circle icons for completed steps
  - Warning triangle for system drives and errors
  - Close (X) icon for modal dialogs
  - Icons loaded from `iced_fonts` crate with Bootstrap font
- **Animated Progress Lines**: Visual workflow progression between steps
  - Horizontal animated lines connecting workflow steps
  - Green color when step is completed
  - Gray color for pending steps
  - Drawn using Iced canvas API for smooth rendering
  - Replaces static text separators (━━━━━━━)
- **Device Selection Modal**: Clicking on the target storage now opens a separate modal view with a table-based device selector
  - Table layout showing Name, Size, Location, and Status columns
  - Warning indicators for system drives (icon-based)
  - Status badges for "Too small", "System drive", and "Source drive"
  - Clean header with close button (icon)
  - Empty state with refresh option
  - Balena Etcher-inspired design
- **New Messages**: `OpenDeviceSelection` and `CloseDeviceSelection` for modal control
- **State Field**: `device_selection_open` to track modal visibility
- **Documentation**: 
  - `docs/FLATTENED_STRUCTURE.md` explaining the new structure
  - Updated `PROJECT_STRUCTURE.md` with flattened organization details

### Changed
- **Icon System**: Replaced text emoji with professional Bootstrap icon font
  - Header: Lightning bolt icon instead of ⚡ emoji
  - Image selection: File icon and plus-circle instead of 📁 and +
  - Target selection: Device icon instead of 💾 emoji
  - Flash button: Lightning icon instead of ⚡ emoji
  - Success states: Check-circle icon instead of ✓ text
  - Warning states: Warning triangle icon instead of ⚠ text
  - All icons use consistent styling and theming
- **Step Indicator**: Enhanced visual workflow progression
  - Icons change based on completion state (icon → check-circle)
  - Icons colored green when completed
  - Animated progress lines replace text separators
  - Smooth canvas-based line rendering
- **Project Structure Flattened**: Eliminated nested `mod.rs` files
  - `src/command/mod.rs` → `src/command.rs` (3.8K)
  - `src/message/mod.rs` → `src/message.rs` (2.0K)
  - `src/model/` (3 files) → `src/model.rs` (7.9K)
  - `src/view/` (2 files) → `src/view.rs` (15K)
  - Reduced from 9 files to 6 files
  - Total codebase: ~1,320 lines across 6 files
- **Model Organization**: Combined `DriveInfo`, `ImageInfo`, and `FlashKraft` into single `model.rs` with section markers
- **View Organization**: Combined entry point and components into single `view.rs` with clear section markers
- **Target Selection UI**: Changed from inline drive list to button that opens modal
- **Device Detection**: Automatically refreshes drives when device selector opens
- **Size Formatting**: Smart size display formatting
  - Shows GB for devices >= 1 GB
  - Shows MB for devices >= 1 MB but < 1 GB
  - Shows KB for very small devices (< 1 MB)
  - Applies to image size, target size, and device selector
- **Device Names**: Enhanced device display names
  - Shows vendor and model when available (e.g., "Samsung SSD 970 EVO (nvme0n1)")
  - Falls back to device name if model unavailable
  - Clearer identification of physical devices

### Improved
- **Visual Design**: Modern, professional appearance with icon font
- **User Experience**: Clear visual feedback with animated progress indicators
- **Accessibility**: Consistent icon usage improves visual hierarchy
- **Performance**: Canvas-based rendering for smooth animations
- **Code Navigation**: Clear file names directly map to Elm Architecture concepts
- **Maintainability**: All related code in one place, fewer files to navigate
- **Readability**: Section markers (`// ===`) organize functionality within files
- **Documentation**: Comprehensive explanation of flattened structure benefits
- **Dependencies**: Cleaner dependency graph with no circular dependencies

### Technical Details
- All 15 tests passing
- **Zero warnings** in project code ✓
- No breaking changes to public API
- Release binary size: ~15MB (includes Bootstrap icons font ~128KB)
- Build time: ~1.5 minutes (release, with canvas feature)
- Total lines of code: ~1,510 lines across 6 files
- **Enhanced drive detection**: Uses `block-utils` for comprehensive device enumeration
- **Real device writing**: Direct block device I/O with proper safety checks
- Clean clippy output
- New dependencies:
  - `iced_fonts` v0.1.1 with Bootstrap icons feature
  - `block-utils` v0.11 for block device operations
  - `tokio` v1 for async task execution
  - Iced `canvas` feature enabled for animated lines
- Font file: `fonts/bootstrap-icons.woff2` (128KB)

## Architecture

FlashKraft follows **The Elm Architecture** with a flattened module structure:

```
src/
├── model.rs      (7.9K)  - Complete state definition
├── message.rs    (2.0K)  - All event types
├── update.rs     (7.1K)  - State transition logic
├── view.rs       (16K)   - UI rendering (now with icons & canvas)
├── command.rs    (3.8K)  - Side effects/async operations
└── main.rs       (3.0K)  - Application entry point (font loading)

fonts/
└── bootstrap-icons.woff2 (128K) - Bootstrap Icons font
```

## Benefits of Flattened Structure

1. **Clear Navigation**: Know exactly which file contains what you need
2. **Easier Maintenance**: All related code in one place
3. **Better Learning**: Structure reflects architecture
4. **Simpler Development**: Less context switching, predictable locations
5. **Clean Dependencies**: No circular dependencies, clear ownership

## Icon System

Bootstrap Icons integration provides:

- **Professional Icons**: High-quality vector icons throughout the UI
- **Consistent Design**: All icons from the same icon family
- **Semantic Icons**: 
  - File/document icon for image selection
  - Device/SSD icon for target selection
  - Lightning bolt for flash operation
  - Check circles for completed steps
  - Warning triangles for errors and system drives
- **Dynamic Styling**: Icons change color based on state (green for complete, white for pending)
- **Performance**: Efficient font-based rendering

## Animated Progress Lines

Visual workflow progression features:

- **Canvas Rendering**: Smooth lines drawn with Iced canvas API
- **State-based Coloring**: 
  - Green lines for completed transitions
  - Gray lines for pending transitions
- **Horizontal Lines**: 100px width, 4px height
- **Smooth Animation**: Ready for future animation enhancements
- **Professional Look**: Replaces text-based separators

## Device Selector Features

The new device selector modal provides:

- **Table-based Layout**: Clear columns for Name, Size, Location, Status
- **Comprehensive Device Detection** (via block-utils):
  - Shows ALL block devices (mounted and unmounted)
  - Detects device model and vendor information from sysfs
  - Automatic mount status detection
  - Smart size formatting (GB/MB/KB)
- **Safety Indicators**: 
  - Warning icon (Bootstrap triangle) for system drives
  - "System drive" badge for OS/home partitions
  - "Source drive" badge if image is on this drive
  - "Too small" badge for drives < 1 GB
- **User Experience**:
  - Click target panel to open selector
  - Click device row to select
  - Automatic modal close on selection
  - Close button (X icon) or Cancel to exit
  - Refresh button when no drives detected
- **Responsive Design**: Fixed-width modal (800px) centered on screen

## Real Device Writing

FlashKraft now supports actual OS image writing:

### Safety Features
- **Permission Verification**: Requires root/sudo privileges
- **Automatic Unmounting**: Unmounts devices before writing
- **Device Erasure**: Clears old partition tables
- **Size Verification**: Confirms written bytes match image size
- **Filesystem Sync**: Ensures all data is flushed to disk

### Writing Process
1. Verify image file and target device
2. Check and unmount if device is mounted
3. Erase device (clear partition tables)
4. Write image in 4MB blocks
5. Sync filesystem
6. Verify bytes written

### Requirements
- **Linux**: Full support with block-utils
- **Root Access**: Must run with `sudo ./flashkraft`
- **Device Access**: Target must be a block device (/dev/sdX)

### Usage
```bash
# Run with sudo for device writing
sudo ./flashkraft

# Or from cargo
sudo cargo run --release
```

**Warning**: Writing to the wrong device will destroy data. Always verify the target device before flashing!

## Migration Notes

If you have an existing checkout, the refactoring is transparent:
- All public APIs remain the same
- Tests continue to pass
- Functionality unchanged
- Only internal organization improved

To understand the new structure:
1. Read `docs/FLATTENED_STRUCTURE.md` for detailed rationale
2. Review `PROJECT_STRUCTURE.md` for file responsibilities
3. Each file has clear section markers for navigation

## Future Enhancements

Planned improvements:
- [x] ~~More sophisticated drive detection~~ - ✅ Implemented via `block-utils`
- [x] ~~Real device writing implementation~~ - ✅ Implemented with `block-utils`
- [ ] Real-time progress updates during flashing (via channels)
- [ ] Animated progress line transitions (smooth width animation)
- [ ] More icon animations and transitions
- [ ] Compressed image format support
- [ ] Write verification
- [ ] Multi-device flashing
- [ ] Settings panel
- [ ] Removable device detection (distinguish USB/SD from internal drives)
- [ ] Custom icon colors and themes
- [ ] Device hotplug detection (auto-refresh on insert/remove)

---

**Note**: This is the first documented release. Previous development was tracked in the conversation history.