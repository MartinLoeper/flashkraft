# FlashKraft 🗲

A modern OS image writer application inspired by Balena Etcher and Raspberry Pi Imager, built with Rust and the [Iced](https://github.com/iced-rs/iced) GUI framework.

![FlashKraft](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 🎨 Modern, clean UI inspired by Balena Etcher
- 📁 Support for multiple image formats (ISO, IMG, DMG, ZIP, etc.)
- 💾 Automatic detection of removable drives
- ⚡ Fast and efficient flashing (simulated in current version)
- 🔒 Safe drive selection to prevent accidental data loss
- 🎯 Progress tracking during flash operations
- 🌙 Dark theme by default

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
│  User Interaction  →  Message  →  Update  →  State     │
│                                      ↓                  │
│                                   Command               │
│                                      ↓                  │
│  Async Result  →  Message  →  Update  →  State         │
│                                                         │
│                    State  →  View  →  UI               │
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
│   └── main.rs          # Main application following Elm Architecture
├── Cargo.toml           # Rust dependencies and project metadata
├── README.md            # This file
└── LICENSE              # MIT License
```

### Code Organization in main.rs

```rust
// 1. MODEL (State)
struct FlashKraft { ... }
struct ImageInfo { ... }
struct DriveInfo { ... }

// 2. MESSAGE
enum Message { ... }

// 3. UPDATE
impl Application for FlashKraft {
    fn update(&mut self, message: Message) -> Command<Message> { ... }
}

// 4. VIEW
fn view(&self) -> Element<'_, Message> { ... }

// VIEW HELPERS (keeping view functions focused)
fn view_header() -> Element<...> { ... }
fn view_step_indicator(...) -> Element<...> { ... }
fn view_main_section(...) -> Element<...> { ... }

// COMMANDS (Side Effects)
async fn select_image_file() -> ... { ... }
async fn load_drives() -> ... { ... }
async fn flash_image() -> ... { ... }
```

## Dependencies

- **iced** (0.12): Cross-platform GUI framework
- **tokio**: Async runtime for handling side effects
- **rfd**: Native file dialog for file selection
- **sysinfo**: System information for drive detection

## Current Limitations

This is a demonstration project showing the Elm Architecture. The actual flashing functionality is simulated. For production use, you would need to:

1. Implement actual block-level device writing
2. Add proper drive detection (removable vs. fixed)
3. Implement verification after writing
4. Add proper error handling and recovery
5. Handle permissions (requires root/admin on most systems)
6. Add safety checks to prevent writing to system drives

## Future Enhancements

- [ ] Real image writing functionality
- [ ] Image verification after writing
- [ ] Support for compressed images
- [ ] Drive format detection
- [ ] Multi-threaded writing for better performance
- [ ] Persistent settings
- [ ] Custom theme support
- [ ] Internationalization (i18n)

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

- Inspired by [Balena Etcher](https://www.balena.io/etcher/)
- Built with [Iced](https://github.com/iced-rs/iced)
- Follows [The Elm Architecture](https://guide.elm-lang.org/architecture/)

## Screenshots

*Note: Add screenshots of your application here*

---

**⚠️ Warning**: This is a demonstration project. Do not use it to write to drives containing important data. Always backup your data before performing disk operations.