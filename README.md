# FlashKraft 🗲

[![Crates.io](https://img.shields.io/crates/v/flashkraft)](https://crates.io/crates/flashkraft)
[![Documentation](https://docs.rs/flashkraft/badge.svg)](https://docs.rs/flashkraft)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Release](https://github.com/sorinirimies/flashkraft/actions/workflows/release.yml/badge.svg)](https://github.com/sorinirimies/flashkraft/actions/workflows/release.yml)
[![CI](https://github.com/sorinirimies/flashkraft/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/flashkraft/actions/workflows/ci.yml)

A lightning fast, lightweight app size and memory footprint, no Electron bloat, OS Imager application built with Rust and the [Iced](https://github.com/iced-rs/iced) GUI framework.
## Preview
![flashkraft_demo](https://github.com/user-attachments/assets/76549cb3-a65e-4a99-b638-1aac6d50c553)

## Features

- 🎨 Small app size, extremely low memory footprint, Rust compiled code has C like and even better speed, no Electron runtime bloat needed, unlike Balena-Etcher needs to, [see Balena Etcher Electron bloat discussions](https://forums.balena.io/t/why-is-etcher-so-bloated/368873)
- 🎨 21 beautiful Iced themes to choose from
- 📁 Support for multiple image formats (ISO, IMG, DMG, ZIP, etc.)
- 💾 Automatic detection of removable drives
- ⚡ Fast and efficient flashing (simulated in current version)
- 🔒 Safe drive selection to prevent accidental data loss
- 🎯 Progress tracking during flash operations

### Code Examples

Working Rust examples demonstrating FlashKraft's architecture and patterns:

```bash
cargo run --example basic_usage       # Full FlashKraft application
cargo run --example custom_theme      # Theme system showcase
```

See [examples/README.md](examples/README.md) for more details.

## The Elm Architecture

FlashKraft is built using **The Elm Architecture (TEA)**, which Iced embraces as the natural approach for building interactive applications. This architecture provides a simple, scalable pattern for managing application state and side effects.

### Core Concepts

The Elm Architecture consists of four main components:

#### 1. **Model (State)**

The Model represents the complete state of the application at any given moment:

```rust
struct FlashKraft {
    selected_image: Option<ImageInfo>,      // Currently selected image file
    selected_target: Option<DriveInfo>,     // Currently selected target drive
    available_drives: Vec<DriveInfo>,       // List of detected drives
    flash_progress: Option<f32>,            // Flash progress (0.0 to 1.0)
    error_message: Option<String>,          // Error message if any
}
```

The state is immutable and only changes through the `update` function. This makes the application predictable and easy to reason about.

#### 2. **Message**

Messages represent all possible events that can occur in the application:

```rust
enum Message {
    // User interactions
    SelectImageClicked,
    RefreshDrivesClicked,
    TargetDriveClicked(DriveInfo),
    FlashClicked,
    ResetClicked,
    CancelClicked,

    // Async results
    ImageSelected(Option<PathBuf>),
    DrivesRefreshed(Vec<DriveInfo>),
    FlashProgress(f32),
    FlashCompleted(Result<(), String>),
}
```

Messages are the only way to trigger state changes. They can come from:
- User interactions (button clicks, etc.)
- Async operations (file selection, drive detection)
- Timers or subscriptions

#### 3. **Update Logic**

The `update` function is the heart of the application. It's a pure function that:
- Takes the current state and a message
- Returns a new state and optionally a `Command` for side effects

```rust
fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        Message::SelectImageClicked => {
            // Trigger async file selection
            Command::perform(select_image_file(), Message::ImageSelected)
        }
        Message::ImageSelected(maybe_path) => {
            // Update state with selected image
            self.selected_image = maybe_path.map(ImageInfo::from_path);
            Command::none()
        }
        // ... handle other messages
    }
}
```

This separation ensures:
- State transitions are predictable
- Side effects are explicit
- Logic is easy to test

#### 4. **View Logic**

The `view` function is a pure function that:
- Takes the current state
- Returns a description of the UI

```rust
fn view(&self) -> Element<'_, Message> {
    if let Some(progress) = self.flash_progress {
        view_flashing(progress)
    } else if let Some(error) = &self.error_message {
        view_error(error)
    } else {
        view_main(self)
    }
}
```

The view is declarative - you describe *what* the UI should look like based on state, not *how* to update it.

### Benefits of The Elm Architecture

1. **Predictability**: State flows in one direction, making behavior easy to predict
2. **Testability**: Pure functions are easy to unit test
3. **Debuggability**: All state changes go through `update`, making it easy to log and debug
4. **Maintainability**: Clear separation of concerns makes code easy to understand and modify
5. **Type Safety**: Rust's type system ensures correctness at compile time

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│  User Interaction  →  Message  →  Update  →  State      │
│                                      ↓                  │
│                                   Command               │
│                                      ↓                  │
│  Async Result  →  Message  →  Update  →  State          │
│                                                         │
│                    State  →  View  →  UI                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Building and Running

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build

```bash
# Clone the repository
git clone https://github.com/yourusername/flashkraft.git
cd flashkraft

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Development

```bash
# Run in debug mode (faster compilation, slower runtime)
cargo run

# Check code without building
cargo check

# Run with backtrace on errors
RUST_BACKTRACE=1 cargo run
```

## Usage

1. **Select Image**: Click the "+" button to select an OS image file (ISO, IMG, DMG, etc.)
2. **Select Target**: Choose a target drive from the list of detected drives
3. **Flash**: Click the "Flash!" button to start writing the image to the drive
4. **Wait**: Monitor the progress bar until completion
5. **Done**: Safely remove the drive when prompted

## Project Structure

```
flashkraft/
├── src/
│   ├── main.rs                  # Application entry point
│   ├── view.rs                  # View orchestration
│   ├── core/                    # Core application logic (Elm Architecture)
│   │   ├── mod.rs
│   │   ├── state.rs             # Application state (Model)
│   │   ├── message.rs           # Message definitions
│   │   ├── update.rs            # Update logic
│   │   ├── storage.rs           # Persistent storage
│   │   ├── flash_subscription.rs # Flash operation monitoring
│   │   └── commands/            # Async commands (side effects)
│   │       ├── mod.rs
│   │       ├── file_selection.rs
│   │       └── drive_detection.rs
│   ├── domain/                  # Domain models (business entities)
│   │   ├── mod.rs
│   │   ├── drive_info.rs
│   │   └── image_info.rs
│   ├── components/              # UI components
│   │   ├── mod.rs
│   │   ├── animated_progress.rs
│   │   ├── device_selector.rs
│   │   ├── header.rs
│   │   ├── progress_line.rs
│   │   ├── selection_panels.rs
│   │   ├── status_views.rs
│   │   ├── step_indicators.rs
│   │   └── theme_selector.rs
│   └── utils/                   # Utility modules
│       ├── mod.rs
│       ├── icons_bootstrap_mapper.rs
│       └── logger.rs
├── .github/
│   └── workflows/
│       ├── ci.yml               # Continuous Integration
│       └── release.yml          # Release automation
├── Cargo.toml                   # Rust dependencies and project metadata
├── README.md                    # This file
└── LICENSE                      # MIT License
```

## Dependencies

- **iced** (0.13): Cross-platform GUI framework
- **iced_fonts**: Bootstrap icons for the UI
- **rfd**: Native file dialog for file selection
- **sysinfo**: System information for drive detection
- **sled**: Embedded database for theme persistence
- **futures**: Async utilities for subscriptions

## Architecture Highlights

- **26 modules**: Well-organized codebase
- **4 layers**: Domain, Core, Components, View
- **22 tests**: 100% passing
- **0 warnings**: Clean clippy and rustfmt
- **Elm Architecture**: Pure functional state management
- **Type-safe**: Leveraging Rust's type system

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. When contributing, please:

1. Follow the Elm Architecture pattern
2. Keep functions pure where possible
3. Use meaningful message names
4. Add comments for complex logic
5. Test your changes

## Learning Resources

- [Iced Documentation](https://docs.rs/iced/)
- [The Elm Architecture Guide](https://guide.elm-lang.org/architecture/)
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [Rust Book](https://doc.rust-lang.org/book/)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced)
- Follows [The Elm Architecture](https://guide.elm-lang.org/architecture/)
