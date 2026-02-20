# FlashKraft 🗲

[![Crates.io](https://img.shields.io/crates/v/flashkraft)](https://crates.io/crates/flashkraft)
[![Documentation](https://docs.rs/flashkraft/badge.svg)](https://docs.rs/flashkraft)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Release](https://github.com/sorinirimies/flashkraft/actions/workflows/release.yml/badge.svg)](https://github.com/sorinirimies/flashkraft/actions/workflows/release.yml)
[![CI](https://github.com/sorinirimies/flashkraft/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/flashkraft/actions/workflows/ci.yml)

A lightning-fast, lightweight OS image writer built entirely in Rust with the [Iced](https://github.com/iced-rs/iced) GUI framework. No Electron, no shell scripts, no external tooling — pure Rust from UI to block device.

## Preview

![flashkraft_demo](https://github.com/user-attachments/assets/76549cb3-a65e-4a99-b638-1aac6d50c553)

## Features

- ⚡ **Pure-Rust flash engine** — no `dd`, no bash scripts; writes directly to the block device using `std::fs`, `nix` ioctls, and `sha2` verification
- 🔒 **Write verification** — SHA-256 of the source image is compared against a read-back of the device after every flash
- 🔄 **Partition table refresh** — `BLKRRPART` ioctl ensures the kernel picks up the new partition layout immediately, so the USB boots first time
- 🧲 **Lazy unmount** — all partitions are cleanly detached via `umount2(MNT_DETACH)` before writing
- 🎨 **21 beautiful Iced themes** to choose from, persisted across sessions
- 📁 **Multiple image formats** — ISO, IMG, DMG, ZIP, and more
- 💾 **Automatic drive detection** — removable drives refreshed on demand
- 🛡️ **Safe drive selection** — system drives flagged, oversized drives warned, read-only drives blocked
- 🎯 **Real-time progress** — stage-aware progress bar with live MB/s speed display
- 🪶 **Tiny footprint** — Rust-compiled binary, C-like memory usage, no Electron runtime

## How flashing works

FlashKraft uses a **self-elevating pure-Rust helper** pattern. When you click Flash:

```
Main process (GUI)
  └─ pkexec /path/to/flashkraft --flash-helper <image> <device>
       └─ Runs as root, pure Rust, no shell
            1. UNMOUNTING  — reads /proc/mounts, calls umount2(MNT_DETACH) per partition
            2. WRITING     — streams image → block device in 4 MiB chunks, emits PROGRESS lines
            3. SYNCING     — fsync(fd) + sync() to flush all kernel write-back caches
            4. REREADING   — BLKRRPART ioctl so the kernel sees the new partition table
            5. VERIFYING   — SHA-256(image) == SHA-256(device[0..image_size])
            6. DONE        — GUI shows success
```

The same binary is re-executed with elevated privileges via `pkexec` — no separate helper binary needs to be installed. All output (progress, logs, errors) is written to stdout as structured lines that the GUI reads in real time.

### Why not `dd`?

| | `dd` approach | FlashKraft |
|---|---|---|
| Shell dependency | ✗ requires bash, coreutils | ✓ pure Rust |
| Progress format | `\r`-terminated, locale-dependent | ✓ structured `PROGRESS:bytes:speed` |
| Speed unit handling | ✗ kB/s / MB/s / GB/s mixed | ✓ always normalised to MB/s |
| Write verification | ✗ none | ✓ SHA-256 read-back |
| Partition table refresh | ✗ not done | ✓ `BLKRRPART` ioctl |
| Error reporting | ✗ exit code only | ✓ `ERROR:<message>` on every failure path |

### Why not `nusb` or `usbd_bulk_only_transport`?

USB flash drives are **USB Mass Storage Class** devices. The kernel's `usb-storage` driver speaks the BOT + SCSI protocol to them and exposes the result as a plain block device (`/dev/sdX`). Writing an OS image needs the **block device interface**, not USB protocol programming.

- `nusb` — USB protocol library (user-space driver for *non-standard* USB devices; MSC is explicitly out of scope per its own docs)
- `usbd_bulk_only_transport` — embedded device-side firmware crate (runs *on* the USB device, not the host)

## The Elm Architecture

FlashKraft is built using **The Elm Architecture (TEA)**, which Iced embraces as its natural pattern for interactive applications.

### Core Concepts

#### 1. Model (State)

```rust
struct FlashKraft {
    selected_image: Option<ImageInfo>,   // Currently selected image file
    selected_target: Option<DriveInfo>,  // Currently selected target drive
    available_drives: Vec<DriveInfo>,    // Detected drives
    flash_progress: Option<f32>,         // Progress 0.0–1.0
    flash_bytes_written: u64,            // Bytes written so far
    flash_speed_mb_s: f32,              // Current transfer speed
    error_message: Option<String>,       // Error message if any
    flashing_active: bool,              // Subscription guard
    flash_cancel_token: Arc<AtomicBool>, // Cancellation signal
}
```

State is immutable between `update` calls — all changes flow through messages.

#### 2. Messages

```rust
enum Message {
    // User interactions
    SelectImageClicked,
    TargetDriveClicked(DriveInfo),
    FlashClicked,
    CancelFlash,
    ResetClicked,

    // Async results
    ImageSelected(Option<PathBuf>),
    DrivesRefreshed(Vec<DriveInfo>),
    FlashProgressUpdate(f32, u64, f32),
    FlashCompleted(Result<(), String>),
    ThemeChanged(Theme),
}
```

#### 3. Update Logic

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::FlashClicked => {
            // Reset cancel token, activate subscription
            state.flash_cancel_token = Arc::new(AtomicBool::new(false));
            state.flashing_active = true;
            Task::none()
        }
        Message::FlashProgressUpdate(progress, bytes, speed) => {
            state.flash_progress = Some(progress);
            state.flash_bytes_written = bytes;
            state.flash_speed_mb_s = speed;
            Task::none()
        }
        // ...
    }
}
```

#### 4. View Logic

Pure function from state to UI — no side effects, no mutation.

#### 5. Subscriptions

When `flashing_active` is true, a subscription spawns the privileged helper and streams [`FlashProgress`] events back to `update`.

### Data Flow

```
User Action → Message → Update → State
                            ↓
                         Task/Subscription
                            ↓
                    Async Result → Message → Update → State
                                                ↓
                                             View → UI
```

## Building and Running

### Prerequisites

- Rust 1.70 or later
- `pkexec` (part of `polkit`, available on all major Linux distributions)

### Build

```bash
git clone https://github.com/sorinirimies/flashkraft.git
cd flashkraft

cargo build --release
cargo run --release
```

### Development

```bash
# Debug build (faster compilation)
cargo run

# Run tests
cargo test

# Check without building
cargo check

# With backtraces
RUST_BACKTRACE=1 cargo run
```

### Examples

```bash
cargo run --example basic_usage   # Full application
cargo run --example custom_theme  # Theme showcase
```

## Usage

1. **Select Image** — click the `+` button and choose an ISO, IMG, or DMG file
2. **Select Drive** — pick the target USB or SD card from the detected drives list
3. **Flash** — click **Flash!**; authenticate with `pkexec` when prompted
4. **Wait** — the progress bar shows live stage, bytes written, and MB/s
5. **Done** — verification passes automatically; safely remove the drive

## Project Structure

```
flashkraft/
├── src/
│   ├── main.rs                       # Entry point; detects --flash-helper mode
│   ├── lib.rs                        # Library root
│   ├── view.rs                       # View orchestration
│   ├── core/                         # Core application logic (Elm Architecture)
│   │   ├── mod.rs
│   │   ├── state.rs                  # Application state (Model) + Elm methods
│   │   ├── message.rs                # All message variants
│   │   ├── update.rs                 # State transition logic
│   │   ├── storage.rs                # Persistent theme storage (sled)
│   │   ├── flash_helper.rs           # ★ Pure-Rust privileged flash engine
│   │   ├── flash_writer.rs           # ★ Protocol types, line parser, speed normaliser
│   │   ├── flash_subscription.rs     # ★ Iced Subscription — spawns helper, streams events
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── file_selection.rs     # Async native file dialog (rfd)
│   │       └── drive_detection.rs    # Async drive enumeration (/sys/block)
│   ├── domain/                       # Domain models
│   │   ├── mod.rs
│   │   ├── constraints.rs            # Drive/image compatibility checks
│   │   ├── drive_info.rs
│   │   └── image_info.rs
│   ├── components/                   # UI components
│   │   ├── animated_progress.rs
│   │   ├── device_selector.rs
│   │   ├── header.rs
│   │   ├── progress_line.rs
│   │   ├── selection_panels.rs
│   │   ├── status_views.rs
│   │   ├── step_indicators.rs
│   │   └── theme_selector.rs
│   └── utils/
│       ├── icons_bootstrap_mapper.rs
│       └── logger.rs                 # debug_log!, flash_debug!, status_log! macros
├── examples/
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── Cargo.toml
├── CHANGELOG.md
└── LICENSE
```

Items marked ★ form the flash pipeline and are described in detail above.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `iced` | 0.13 | Cross-platform GUI framework (Elm Architecture) |
| `iced_fonts` | 0.1 | Bootstrap icon font |
| `rfd` | 0.15 | Native file/folder dialogs |
| `sysinfo` | 0.30 | Drive enumeration |
| `nix` | 0.29 | `umount2`, `BLKRRPART` ioctl, `fsync` |
| `sha2` | 0.10 | SHA-256 write verification |
| `futures` | 0.3 | Async channel primitives for the subscription |
| `futures-timer` | 3.0 | Non-blocking delay in the subscription poll loop |
| `sled` | 0.34 | Embedded key-value store for theme persistence |
| `dirs` | 5.0 | XDG data directory resolution |

## Architecture Highlights

- **Pure-Rust flash engine** — zero shell scripts or external binaries
- **94 unit tests** — all passing; covers parsing, pipeline validation, I/O round-trips, SHA-256 hashing, partition name detection, and protocol simulation
- **5-stage verified pipeline** — Unmount → Write → Sync → BLKRRPART → SHA-256 verify
- **Self-elevating helper** — single binary, no install-time setup beyond `polkit`
- **Elm Architecture** — unidirectional data flow, pure update/view functions
- **0 warnings** — clean `cargo build` and `cargo test`

## Contributing

Contributions are welcome! Please:

1. Follow the Elm Architecture pattern — all state changes via `update`
2. Keep functions pure where possible
3. Add unit tests for any new logic, especially in `flash_helper.rs` and `flash_writer.rs`
4. Run `cargo test` and `cargo clippy` before opening a PR

## Learning Resources

- [Iced Documentation](https://docs.rs/iced/)
- [The Elm Architecture Guide](https://guide.elm-lang.org/architecture/)
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [nix crate](https://docs.rs/nix/) — POSIX ioctls and syscalls from Rust

## License

MIT — see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced)
- Follows [The Elm Architecture](https://guide.elm-lang.org/architecture/)
- Flash pipeline design inspired by [Balena Etcher](https://github.com/balena-io/etcher)