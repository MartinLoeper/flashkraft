# FlashKraft Examples

This directory contains working examples demonstrating FlashKraft's features and architecture.

## Available Examples

### 1. Basic Usage (`basic_usage.rs`)

Run the complete FlashKraft application with all features enabled.

```bash
cargo run --example basic_usage
```

**What it demonstrates:**
- Full OS image writer functionality
- Image file selection (ISO, IMG, DMG, ZIP)
- Automatic removable drive detection
- Device selection interface
- Flash progress tracking with speed and ETA
- Safe drive selection to prevent data loss
- All 21 beautiful themes
- The complete Elm Architecture pattern

**Usage:**
1. Click "Select Image" to choose an OS image file
2. Click "Select Target" to choose a USB/SD drive
3. Click "Flash!" to start the flashing process
4. Click the theme button (🎨) to explore different themes

---

### 2. Custom Theme (`custom_theme.rs`)

Showcase FlashKraft's powerful theme system with a pre-selected theme.

```bash
cargo run --example custom_theme
```

**What it demonstrates:**
- Runtime theme switching
- 21 built-in Iced themes
- Persistent theme storage across sessions
- Consistent theming across all UI components
- Theme-aware progress bars and animations
- How themes affect the entire application

**Available Themes:**

**Dark Themes:**
- Dark (default)
- Dracula
- Nord
- Solarized Dark
- Gruvbox Dark
- Catppuccin Mocha
- Tokyo Night
- Kanagawa Wave
- Moonfly
- Nightfly
- Oxocarbon

**Light Themes:**
- Light
- Solarized Light
- Gruvbox Light
- Catppuccin Latte
- Tokyo Night Light
- Kanagawa Lotus

**And more variants...**

**Usage:**
1. The app starts with Tokyo Night theme pre-selected
2. Click the theme button (🎨) in the top-right corner
3. Select any theme from the picker
4. Watch the entire UI update instantly
5. Your theme preference is saved automatically

---

## Architecture Highlights

Both examples use the real FlashKraft application, which follows **The Elm Architecture**:

### The Elm Architecture (TEA)

FlashKraft's architecture consists of four core components:

1. **Model** (`FlashKraft` struct)
   - Complete application state
   - Immutable data structures
   - Single source of truth

2. **Message** (Events)
   - User interactions (button clicks, file selections)
   - Async operation results (drive detection, flash progress)
   - System events (animation ticks)

3. **Update** (State transitions)
   - Pure function: `(State, Message) → (State, Command)`
   - All state changes flow through update
   - Side effects as Commands/Tasks

4. **View** (UI rendering)
   - Pure function: `State → UI`
   - Declarative UI description
   - Automatic re-rendering on state changes

### Data Flow

```
User Action → Message → Update → New State → View → UI
                          ↓
                       Command
                          ↓
                   Async Task → Message
```

### Module Structure

```
src/
├── core/           # Core application logic (Elm Architecture)
│   ├── state.rs    # Application state (Model)
│   ├── message.rs  # Message definitions
│   ├── update.rs   # Update logic
│   └── commands/   # Async commands (side effects)
├── domain/         # Domain models
│   ├── drive_info.rs
│   └── image_info.rs
├── components/     # UI components
│   ├── header.rs
│   ├── device_selector.rs
│   ├── animated_progress.rs
│   └── theme_selector.rs
└── view.rs         # Main view orchestration
```

---

## Building and Running

### Prerequisites

- Rust 1.70 or later
- Cargo

### Run an Example

```bash
# Basic usage example
cargo run --example basic_usage

# Theme system example
cargo run --example custom_theme
```

### Build Examples

```bash
# Build all examples
cargo build --examples

# Build specific example
cargo build --example basic_usage
```

---

## Learning Path

We recommend exploring the examples in this order:

1. **Start with `basic_usage.rs`**
   - Understand the complete application flow
   - See how image selection and device detection work
   - Experience the flash operation
   - Explore different themes

2. **Then try `custom_theme.rs`**
   - Focus on the theme system
   - See how themes affect all components
   - Experiment with theme persistence

3. **Dive into the source code**
   - Read `src/core/state.rs` for the Model
   - Check `src/core/message.rs` for all Messages
   - Study `src/core/update.rs` for Update logic
   - Explore `src/view.rs` for View composition

---

## Key Takeaways

### The Elm Architecture Benefits

- **Predictable**: All state changes go through update
- **Testable**: Pure functions are easy to test
- **Maintainable**: Clear separation of concerns
- **Scalable**: Easy to add new features
- **Debuggable**: Message log shows all state transitions

### Best Practices Demonstrated

- Immutable state management
- Declarative UI composition
- Type-safe message handling
- Async operations as Tasks
- Component-based UI structure
- Persistent user preferences
- Responsive design patterns

---

## Resources

- [Iced Framework](https://github.com/iced-rs/iced)
- [The Elm Architecture](https://guide.elm-lang.org/architecture/)
- [FlashKraft Repository](https://github.com/sorinirimies/flashkraft)
- [FlashKraft Documentation](https://docs.rs/flashkraft)

---

## Contributing

Found an issue or have an idea for a new example? Please open an issue or submit a pull request!