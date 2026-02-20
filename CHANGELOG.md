# Changelog

All notable changes to this project will be documented in this file.

## 0.4.0 - 2025-11-14

### ⚡ Pure-Rust Flash Engine (breaking change to internals)

The entire flash pipeline has been rewritten in pure Rust. No shell scripts,
no `dd`, no `bash` — a single binary does everything.

#### How it works

The main binary re-executes itself with elevated privileges via `pkexec`:

```
pkexec /path/to/flashkraft --flash-helper <image_path> <device_path>
```

`main.rs` detects `--flash-helper` before touching any GUI code and
dispatches to the new `flash_helper` module. The helper writes structured
progress lines to stdout; the Iced subscription reads them in real time.

#### New: `src/core/flash_helper.rs`

Pure-Rust privileged flash engine implementing a five-stage pipeline:

| Stage | What happens |
|-------|-------------|
| UNMOUNTING | Reads `/proc/mounts`; calls `libc::umount2(MNT_DETACH)` for every partition |
| WRITING | Streams image → block device in 4 MiB chunks; emits `PROGRESS:bytes:speed` |
| SYNCING | `libc::fsync(fd)` + `libc::sync()` to flush all kernel write-back caches |
| REREADING | `BLKRRPART` ioctl (`nix::ioctl_none!`) forces kernel to re-read partition table |
| VERIFYING | SHA-256 of source image compared against read-back of device |

#### New: `src/core/flash_writer.rs` — protocol types and parser

- `FlashStage` enum with `Display` for all five stages
- `ScriptLine` enum covering all structured output lines
- `ScriptLine::Progress(u64, f32)` — primary progress format (`PROGRESS:bytes:speed_mb_s`)
- `parse_script_line()` — parses one line of helper stdout
- `parse_dd_progress()` — legacy `dd status=progress` parser (kept for compat)
- `parse_speed_to_mb_s()` — normalises `kB/s`, `MB/s`, `GB/s`, `GiB/s`, `B/s` → MB/s

#### Updated: `src/core/flash_subscription.rs`

- Spawns `pkexec <self_exe> --flash-helper <image> <device>` instead of a bash script
- Single reader thread on one file descriptor (no more stdout/stderr race)
- Splits on both `\r` and `\n` so legacy dd carriage-return updates work too
- Handles `ScriptLine::Progress` (primary) and `ScriptLine::DdProgress` (legacy)
- Removed all temporary script file creation/cleanup

#### Updated: `src/main.rs`

- Detects `--flash-helper <image> <device>` arguments before starting the GUI
- Dispatches to `flash_helper::run()` and exits; GUI code is never touched

### 🐛 Bug Fixes

- **USB not booting after flash** — root cause was the missing `BLKRRPART` ioctl;
  the kernel kept its stale partition table cache until the device was physically
  re-inserted. Now fixed unconditionally after every successful write.
- **Speed display wrong for slow devices** — `kB/s` values from `dd` were
  previously displayed as if they were `MB/s` (e.g. `500 kB/s` showed as
  `500 MB/s`). `parse_speed_to_mb_s` now normalises all units correctly.
- **Silent partial-write failures** — the old `set -e` bash script would exit
  without an error message if any sub-command failed. All failure paths now
  emit an explicit `ERROR:<message>` line.
- **Unmounting broken for NVMe / eMMC** — glob patterns like `${DEVICE}p*[0-9]`
  did not handle `nvme0n1p1` or `mmcblk0p1` correctly. The new Rust
  `is_partition_of()` function handles all Linux naming conventions.

### ➕ Added dependencies

- `nix = { version = "0.29", features = ["fs", "ioctl", "mount"] }` — `umount2`, `BLKRRPART`, `fsync`
- `sha2 = "0.10"` — SHA-256 write verification

### 🧪 Tests

Test count raised from **35 → 94** (all passing):

- `flash_helper`: 16 tests — partition naming, `/proc/mounts` parsing,
  `write_image` round-trip, SHA-256 hashing, pipeline validation, end-to-end
  temp-file flash
- `flash_writer`: 57 tests — speed normalisation, `dd` progress parsing,
  `ScriptLine` parsing for all variants, full protocol simulations (success,
  write failure, verification failure)
- `flash_subscription`: 4 tests — `FlashProgress` clone/debug, subscription
  ID determinism

### 📝 Documentation

- Module-level doc-comments rewritten for `flash_helper.rs`,
  `flash_writer.rs`, and `flash_subscription.rs`
- README updated: new architecture diagram, dependency table, "Why not nusb?"
  section, updated project structure and feature list

**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.5...v0.4.0

## 0.3.5 - 2025-11-13
### 🔧 Chores
- chore: bump version to 0.3.5
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.4...v0.3.5
## 0.3.4 - 2025-11-13
### 📦 Other Changes
- Simplify is_source_drive mount point check logic
- Run CI and release workflows only on Ubuntu
### 🔧 Chores
- chore: bump version to 0.3.4
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.3...v0.3.4
## 0.3.3 - 2025-11-13
### ➕ Added
- Add drive/image compatibility checks and warnings
### 📦 Other Changes
- Remove push and release recipes from justfile and backup
### 🔧 Chores
- chore: bump version to 0.3.3
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.2...v0.3.3
## 0.3.2 - 2025-10-30
### ➕ Added
- Add repository link, categories, and keywords to Cargo.toml
### 🔧 Chores
- chore: bump version to 0.3.2
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.1...v0.3.2
## 0.3.1 - 2025-10-27
### 📦 Other Changes
- - Fix race condition where flash progress updates continued to display
- Use theme_selector_right in status views
### 🔧 Chores
- chore: bump version to 0.3.1
- chore: bump version to 0.3.1
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.3.0...v0.3.1
## 0.3.0 - 2025-10-27
### 📦 Other Changes
- Clarify Electron bloat comparison in features section
### 🔧 Chores
- chore: bump version to 0.3.0
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.9...v0.3.0
## 0.2.9 - 2025-10-26
### 📦 Other Changes
- Reduce progress line and glow thickness for sleeker look
- bump version
### 🔧 Chores
- chore: bump version to 0.2.9
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.6...v0.2.9
## 0.2.6 - 2025-10-26
### ➕ Added
- Add cancellation support to flash operation
### 🔧 Chores
- chore: bump version to 0.2.6
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.5...v0.2.6
## 0.2.5 - 2025-10-25
### 🐛 Bug Fixes
- Fix cargo install command in release changelog generation
### 🔧 Chores
- chore: bump version to 0.2.5
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.4...v0.2.5
## 0.2.4 - 2025-10-25
### 🔧 Chores
- chore: bump version to 0.2.4
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.3...v0.2.4
## 0.2.3 - 2025-10-25
### 🔄 Updated
- Update README.md
### 🔧 Chores
- chore: bump version to 0.2.2
- chore: bump version to 0.2.3
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.1...v0.2.3
## 0.2.1 - 2025-10-25
### ➕ Added
- Add no_run and example variables to doc tests in logger macros
### 🔧 Chores
- chore: bump version to 0.2.1
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.2.0...v0.2.1
## 0.2.0 - 2025-10-25
### ♻️ Refactor
- Refactor examples and project structure for FlashKraft library
### 🔧 Chores
- chore: bump version to 0.2.0
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.9...v0.2.0
## 0.1.9 - 2025-10-25
### 🔄 Updated
- Update README.md
### 🔧 Chores
- chore: bump version to 0.1.8
- chore: bump version to 0.1.8
- chore: bump version to 0.1.9
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.7...v0.1.9
## 0.1.7 - 2025-10-25
### 📦 Other Changes
- Remove Quick Start example from release notes generation
### 🔧 Chores
- chore: bump version to 0.1.7
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.6...v0.1.7
## 0.1.6 - 2025-10-25
### 📦 Other Changes
- Adds VHS demo for Elm Architecture
### 🔄 Updated
- Update README.md
- Update README.md
- Update README.md
- Update README.md, cleanup
- Update README.md
### 🔧 Chores
- chore: bump version to 0.1.6
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.5...v0.1.6
## 0.1.5 - 2025-10-24
### 📦 Other Changes
- Scale animation speed based on transfer rate
### 🔧 Chores
- chore: bump version to 0.1.5
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.4...v0.1.5
## 0.1.4 - 2025-10-24
### 📦 Other Changes
- Remove screenshots section from README and simplify header color
### 🔧 Chores
- chore: bump version to 0.1.4
**Full Changelog**: https://github.com/sorinirimies/flashkraft/compare/v0.1.3...v0.1.4
## 0.1.3 - 2025-10-24
### ♻️ Refactor
- Refactor Elm Architecture methods into core/state.rs
### ➕ Added
- Add initial project structure with Elm Architecture and real device
- Add theme picker and support for dynamic themes Add theme picker and
- Add CI, release workflow, examples, VHS demos, and docs
- Add persistent theme storage using sled
- Add Cargo.lock and update project metadata and .gitignore
### 📦 Other Changes
- Initial commit
- Make device selector height fill available space
- Remove architecture and UI documentation from docs
- Show flash speed and ETA during flashing process
- Restructure codebase into core, domain, components, and utils modules
- Bump version to 0.1.1 and update bump_version.sh script
- Replace changelog with release header and add cliff.toml config
### 🔄 Updated
- Update README to reflect new project structure and dependencies
### 🔧 Chores
- chore: bump version to 0.1.3
