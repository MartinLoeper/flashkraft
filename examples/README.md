# FlashKraft Examples

This directory contains working Rust examples demonstrating FlashKraft's architecture, patterns, and usage of The Elm Architecture with Iced.

## Running Examples

Run any example with:

```bash
cargo run --example <example_name>
```

For example:

```bash
cargo run --example basic_usage
cargo run --example state_machine
cargo run --example custom_theme
```

## Available Examples

### 1. `basic_usage.rs`

**Purpose:** Demonstrates the core concepts of The Elm Architecture (TEA) in a minimal Iced application.

**What you'll learn:**
- Basic Model-Message-Update-View pattern
- Simple state management
- Button interactions and event handling
- Pure functional rendering

**Run:**
```bash
cargo run --example basic_usage
```

**Key concepts:**
- Immutable state
- Message-driven updates
- Declarative UI

---

### 2. `state_machine.rs`

**Purpose:** Shows how to model application states as an enum (state machine pattern).

**What you'll learn:**
- Using enums to represent distinct application states
- State transitions and flow control
- Pattern matching in update and view functions
- Handling multi-step workflows

**Run:**
```bash
cargo run --example state_machine
```

**Key concepts:**
- State machine design
- Exhaustive pattern matching
- Type-safe state transitions
- Progress tracking

---

### 3. `custom_theme.rs`

**Purpose:** Demonstrates theme customization and dynamic theme switching.

**What you'll learn:**
- Working with Iced's theme system
- Switching themes at runtime
- Custom color palettes
- Theming best practices

**Run:**
```bash
cargo run --example custom_theme
```

**Key concepts:**
- Theme definition
- Dynamic theme updates
- UI consistency
- Dark/Light mode support

---

### 4. `async_commands.rs`

**Purpose:** Shows how to handle asynchronous operations using Commands.

**What you'll learn:**
- Using `Command::perform` for async operations
- File system operations
- Error handling with Results
- Loading states and spinners

**Run:**
```bash
cargo run --example async_commands
```

**Key concepts:**
- Async/await with Iced
- Command composition
- Error propagation
- User feedback during async operations

---

### 5. `subscription_progress.rs`

**Purpose:** Demonstrates subscriptions for streaming updates (like progress bars).

**What you'll learn:**
- Creating custom subscriptions
- Streaming progress updates
- Long-running operations
- Cancellation support

**Run:**
```bash
cargo run --example subscription_progress
```

**Key concepts:**
- Subscription pattern
- Stream processing
- Progress tracking
- Background tasks

---

### 6. `file_dialog.rs`

**Purpose:** Shows file selection using native dialogs with `rfd`.

**What you'll learn:**
- Opening native file dialogs
- File filtering by extension
- Async dialog handling
- Path manipulation

**Run:**
```bash
cargo run --example file_dialog
```

**Key concepts:**
- Native platform dialogs
- File type filtering
- User file selection
- Cross-platform compatibility

---

### 7. `drive_detection.rs`

**Purpose:** Demonstrates system drive detection using `sysinfo`.

**What you'll learn:**
- Enumerating system drives
- Detecting removable media
- Drive information extraction
- Filtering and sorting drives

**Run:**
```bash
cargo run --example drive_detection
```

**Key concepts:**
- System information queries
- Drive properties
- Safe drive selection
- Platform differences

---

### 8. `layout_composition.rs`

**Purpose:** Shows how to compose complex layouts with Iced widgets.

**What you'll learn:**
- Container and layout widgets
- Spacing and padding
- Alignment and positioning
- Responsive design

**Run:**
```bash
cargo run --example layout_composition
```

**Key concepts:**
- Widget composition
- Layout patterns
- Spacing and alignment
- Nested containers

---

## Example Structure

Each example follows a consistent structure:

```rust
// 1. Imports
use iced::{Element, Task};

// 2. Model (State)
struct ExampleApp {
    // State fields
}

// 3. Message (Events)
#[derive(Debug, Clone)]
enum Message {
    // Event variants
}

// 4. Update (State transitions)
impl ExampleApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        // Handle messages and update state
    }
}

// 5. View (UI rendering)
impl ExampleApp {
    fn view(&self) -> Element<'_, Message> {
        // Render UI based on state
    }
}

// 6. Main function
fn main() -> iced::Result {
    iced::application(
        "Example Title",
        ExampleApp::update,
        ExampleApp::view,
    )
    .run()
}
```

## Learning Path

We recommend going through the examples in this order:

1. **Start here:** `basic_usage.rs` - Learn the fundamentals
2. `state_machine.rs` - Understand state modeling
3. `custom_theme.rs` - Explore theming
4. `layout_composition.rs` - Master layouts
5. `async_commands.rs` - Handle async operations
6. `file_dialog.rs` - Work with file selection
7. `drive_detection.rs` - Query system information
8. `subscription_progress.rs` - Stream updates

## The Elm Architecture in Examples

All examples demonstrate The Elm Architecture pattern:

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  User Input → Message → Update → State → View      │
│                           ↓                         │
│                        Command                      │
│                           ↓                         │
│          Async Result → Message → Update...         │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Key Principles

1. **Single Source of Truth**
   - All state lives in the Model
   - State is immutable
   - Updates create new state

2. **Unidirectional Data Flow**
   - Messages flow one way
   - State updates are predictable
   - Side effects are explicit

3. **Pure Functions**
   - View is a pure function of state
   - Update describes state transitions
   - No hidden state or mutations

4. **Explicit Side Effects**
   - Commands represent async operations
   - Subscriptions for continuous updates
   - All effects return Messages

## Common Patterns

### Pattern 1: Loading States

```rust
enum State {
    Loading,
    Loaded(Data),
    Error(String),
}
```

### Pattern 2: Optional Selection

```rust
struct Model {
    selected_item: Option<Item>,
}
```

### Pattern 3: Multi-Step Workflow

```rust
enum Step {
    SelectFile,
    SelectTarget,
    Confirm,
    Processing,
    Complete,
}
```

### Pattern 4: Error Handling

```rust
enum Message {
    OperationCompleted(Result<Data, String>),
}
```

## Testing Examples

Examples include tests demonstrating:

- State initialization
- Message handling
- State transitions
- Validation logic

Run tests for all examples:

```bash
cargo test --examples
```

Run tests for a specific example:

```bash
cargo test --example basic_usage
```

## Building for Release

Build optimized examples:

```bash
cargo build --release --examples
```

Run optimized version:

```bash
cargo run --release --example basic_usage
```

## Troubleshooting

### Example won't compile

Make sure you have the correct Rust version:

```bash
rustc --version  # Should be 1.70+
cargo --version
```

Update dependencies:

```bash
cargo update
```

### Window doesn't appear

Some examples create GUI windows. Make sure you're running in a graphical environment.

### File dialogs don't work

The `file_dialog.rs` example requires a desktop environment with native dialog support.

## Additional Resources

- [Iced Documentation](https://docs.rs/iced/)
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [The Elm Guide](https://guide.elm-lang.org/)
- [FlashKraft Main Docs](../README.md)
- [Elm Architecture Docs](../ELM_ARCHITECTURE.md)

## Contributing Examples

Want to add a new example? Great! Please:

1. Follow the structure outlined above
2. Include clear comments explaining key concepts
3. Add tests where appropriate
4. Update this README with your example
5. Keep examples focused on one concept
6. Make them runnable standalone

## Questions?

If you have questions about these examples or The Elm Architecture:

1. Check the [main README](../README.md)
2. Read [ELM_ARCHITECTURE.md](../ELM_ARCHITECTURE.md)
3. Look at the main application in `src/`
4. Open an issue on GitHub

---

Happy learning! 🚀