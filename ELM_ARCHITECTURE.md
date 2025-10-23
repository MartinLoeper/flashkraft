# The Elm Architecture in FlashKraft

This document explains how FlashKraft implements The Elm Architecture (TEA) pattern using the Iced framework.

## What is The Elm Architecture?

The Elm Architecture is a pattern for architecting interactive programs. It provides a simple, predictable way to manage application state and handle side effects. The pattern consists of four core concepts:

1. **Model** - The state of your application
2. **Message** - Events that describe state changes
3. **Update** - A function that processes messages and updates state
4. **View** - A function that renders UI based on state

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ User Action
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         Message                             │
│  (SelectImageClicked, TargetDriveClicked, etc.)             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         Update                              │
│  - Process message                                          │
│  - Update state                                             │
│  - Return Command for side effects                          │
└─────────────────────────────────────────────────────────────┘
                              │
                  ┌───────────┴───────────┐
                  │                       │
                  ▼                       ▼
         ┌────────────────┐      ┌────────────────┐
         │   New State    │      │    Command     │
         │                │      │  (Side Effect) │
         └────────────────┘      └────────────────┘
                  │                       │
                  │                       │
                  ▼                       ▼
         ┌────────────────┐      ┌────────────────┐
         │     View       │      │  Async Task    │
         │   (Render)     │      │  (File Dialog, │
         └────────────────┘      │   Drive Scan)  │
                  │              └────────────────┘
                  │                       │
                  ▼                       │
         ┌────────────────┐               │
         │  User Interface│◄──────────────┘
         └────────────────┘      Result → Message
```

## 1. Model (State)

The Model represents the complete state of your application at any point in time.

### Implementation in FlashKraft

```rust
struct FlashKraft {
    selected_image: Option<ImageInfo>,
    selected_target: Option<DriveInfo>,
    available_drives: Vec<DriveInfo>,
    flash_progress: Option<f32>,
    error_message: Option<String>,
}
```

### Key Principles

- **Immutable**: State is never mutated directly, only replaced
- **Complete**: Contains all information needed to render the UI
- **Type-Safe**: Uses Rust's type system to prevent invalid states

### Supporting Types

```rust
struct ImageInfo {
    path: PathBuf,
    name: String,
    size_mb: f64,
}

struct DriveInfo {
    name: String,
    mount_point: String,
    size_gb: f64,
}
```

These types encapsulate domain concepts and make the state easier to work with.

## 2. Message (Events)

Messages represent all possible events that can occur in your application.

### Implementation in FlashKraft

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

### Message Categories

**1. User Interactions** (synchronous)
- Triggered directly by user actions (button clicks)
- Names end with "Clicked" to indicate user intent
- Example: `SelectImageClicked`

**2. Async Results** (asynchronous)
- Results from Commands (file dialogs, I/O operations)
- Often contain data (path, drives, progress)
- Example: `ImageSelected(Option<PathBuf>)`

### Naming Convention

- Present tense: `FlashClicked` not `FlashWasClicked`
- Descriptive: Message name should explain what happened
- Data-carrying: Include relevant data with the message

## 3. Update (State Transitions)

The Update function is the heart of the Elm Architecture. It's where all state changes happen.

### Signature

```rust
fn update(&mut self, message: Message) -> Command<Message>
```

### Responsibilities

1. **Process Messages**: Match on the message type
2. **Update State**: Modify the application state
3. **Spawn Commands**: Return side effects to be executed
4. **Stay Pure**: No I/O or side effects in update logic

### Implementation Pattern

```rust
fn update(&mut self, message: Message) -> Command<Message> {
    match message {
        // Handle user clicking "Select Image"
        Message::SelectImageClicked => {
            // Don't do I/O here! Return a Command instead
            Command::perform(select_image_file(), Message::ImageSelected)
        }

        // Handle the result of image selection
        Message::ImageSelected(maybe_path) => {
            // Update state
            self.selected_image = maybe_path.map(ImageInfo::from_path);
            self.error_message = None;
            // No side effects needed
            Command::none()
        }

        Message::FlashClicked => {
            if self.selected_image.is_some() && self.selected_target.is_some() {
                self.flash_progress = Some(0.0);
                Command::perform(flash_image(), Message::FlashCompleted)
            } else {
                Command::none()
            }
        }

        // ... handle other messages
    }
}
```

### Best Practices

✅ **Do:**
- Keep update logic pure and testable
- Return Commands for side effects
- Validate state before spawning Commands
- Use exhaustive pattern matching

❌ **Don't:**
- Perform I/O in update
- Block on async operations
- Mutate global state
- Skip error handling

## 4. View (Rendering)

The View function renders the UI based on the current state.

### Signature

```rust
fn view(&self) -> Element<'_, Message>
```

### Principles

- **Pure Function**: Same state → same UI
- **Declarative**: Describe what to show, not how to update
- **Event Binding**: Attach Messages to UI elements

### Implementation Pattern

```rust
fn view(&self) -> Element<'_, Message> {
    // Choose what to show based on state
    if let Some(progress) = self.flash_progress {
        view_flashing(progress)
    } else if let Some(error) = &self.error_message {
        view_error(error)
    } else {
        view_main(self)
    }
}
```

### View Composition

Break down complex views into smaller, focused functions:

```rust
fn view_main(state: &FlashKraft) -> Element<'_, Message> {
    column![
        view_header(),
        view_step_indicator(state),
        view_main_section(state),
    ]
    .spacing(30)
    .into()
}

fn view_image_section(image: &Option<ImageInfo>) -> Element<'_, Message> {
    let content = if let Some(img) = image {
        column![
            text("✓").size(50),
            text(&img.name).size(16),
            text(format!("{:.2} MB", img.size_mb)).size(12),
        ]
    } else {
        column![
            text("+").size(50),
            text("Flash from file").size(16),
        ]
    };

    button(container(content))
        .on_press(Message::SelectImageClicked)
        .into()
}
```

### Event Binding

```rust
button(text("Flash!"))
    .on_press(Message::FlashClicked)  // Emit message on click
```

## 5. Commands (Side Effects)

Commands represent side effects that need to happen outside the update loop.

### Types of Commands

**1. Command::none()**
- No side effects needed
- State update is sufficient

**2. Command::perform()**
- Execute an async operation
- Convert result to a Message

### Implementation Pattern

```rust
async fn select_image_file() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .add_filter("Image Files", &["img", "iso", "dmg"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

// In update:
Message::SelectImageClicked => {
    Command::perform(
        select_image_file(),           // Async function
        Message::ImageSelected         // Wrap result in message
    )
}
```

### Command Composition

You can batch multiple commands:

```rust
Command::batch(vec![
    Command::perform(load_drives(), Message::DrivesRefreshed),
    Command::perform(check_permissions(), Message::PermissionsChecked),
])
```

## Complete Data Flow Example

Let's trace a complete user interaction:

### 1. User Clicks "Select Image"

```
UI → Message::SelectImageClicked → Update
```

### 2. Update Spawns Command

```rust
Message::SelectImageClicked => {
    Command::perform(select_image_file(), Message::ImageSelected)
}
```

### 3. Command Executes Async

```rust
async fn select_image_file() -> Option<PathBuf> {
    // Show file dialog (async)
    // User selects file
    // Return path
}
```

### 4. Command Result → Message

```
Command Result → Message::ImageSelected(Some(path)) → Update
```

### 5. Update Processes Result

```rust
Message::ImageSelected(maybe_path) => {
    self.selected_image = maybe_path.map(ImageInfo::from_path);
    Command::none()
}
```

### 6. View Re-renders

```rust
fn view(&self) -> Element {
    // Sees updated state
    // Shows image name and details
}
```

## Benefits of This Architecture

### 1. Predictability
All state changes flow through `update`, making behavior predictable and easy to trace.

### 2. Testability
Pure functions (update, view) are easy to unit test:

```rust
#[test]
fn test_image_selection() {
    let mut app = FlashKraft::new().0;
    let path = PathBuf::from("test.img");

    app.update(Message::ImageSelected(Some(path.clone())));

    assert!(app.selected_image.is_some());
}
```

### 3. Debuggability
Log every message to understand exactly what happened:

```rust
fn update(&mut self, message: Message) -> Command<Message> {
    println!("Message: {:?}", message);  // Debug
    match message {
        // ...
    }
}
```

### 4. Maintainability
Clear separation of concerns makes code easy to understand and modify.

### 5. Type Safety
Rust's type system ensures correctness:
- Invalid states are unrepresentable
- All message paths must be handled
- Async safety is guaranteed

## Common Patterns

### Pattern 1: Async Task with Progress

```rust
// Message
enum Message {
    FlashClicked,
    FlashProgress(f32),
    FlashCompleted(Result<(), String>),
}

// Update
Message::FlashClicked => {
    self.flash_progress = Some(0.0);
    Command::perform(flash_with_progress(), Message::FlashCompleted)
}

// Command (simplified - would use channels in reality)
async fn flash_with_progress() -> Result<(), String> {
    // Perform operation
    // Update progress via channels or subscriptions
    Ok(())
}
```

### Pattern 2: Conditional Commands

```rust
Message::FlashClicked => {
    if self.can_flash() {
        self.flash_progress = Some(0.0);
        Command::perform(flash_image(), Message::FlashCompleted)
    } else {
        self.error_message = Some("Select image and target".into());
        Command::none()
    }
}
```

### Pattern 3: Chained Commands

```rust
Message::ImageSelected(Some(path)) => {
    self.selected_image = Some(ImageInfo::from_path(path));
    // Automatically refresh drives after selecting image
    Command::perform(load_drives(), Message::DrivesRefreshed)
}
```

### Pattern 4: State Machine

Use enums to represent complex state:

```rust
enum AppState {
    Idle,
    SelectingImage,
    ReadyToFlash,
    Flashing { progress: f32 },
    Complete,
    Error(String),
}

// In view:
match self.state {
    AppState::Idle => view_idle(),
    AppState::Flashing { progress } => view_flashing(progress),
    AppState::Complete => view_complete(),
    // ...
}
```

## Anti-Patterns to Avoid

### ❌ Storing UI State in Model

```rust
// Bad
struct FlashKraft {
    button_hovered: bool,  // UI concern, not app state
}

// Good - Let Iced handle UI state
```

### ❌ Direct I/O in Update

```rust
// Bad
Message::SelectImageClicked => {
    let path = std::fs::read("file.img");  // Blocking I/O!
    self.selected_image = Some(path);
    Command::none()
}

// Good
Message::SelectImageClicked => {
    Command::perform(select_image_file(), Message::ImageSelected)
}
```

### ❌ Mutation Without Messages

```rust
// Bad
fn some_helper(&mut self) {
    self.selected_image = None;  // Hidden state change!
}

// Good - All changes through update
Message::ClearImage => {
    self.selected_image = None;
    Command::none()
}
```

## Advanced Topics

### Subscriptions

For ongoing events (timers, keyboard input):

```rust
fn subscription(&self) -> Subscription<Message> {
    if self.flash_progress.is_some() {
        time::every(Duration::from_millis(100))
            .map(|_| Message::UpdateProgress)
    } else {
        Subscription::none()
    }
}
```

### Custom Commands

Wrap complex async operations:

```rust
fn flash_drive(image: PathBuf, target: PathBuf) -> Command<Message> {
    Command::perform(
        async move {
            // Complex flashing logic
            flash_impl(image, target).await
        },
        Message::FlashCompleted
    )
}
```

## Conclusion

The Elm Architecture provides a robust foundation for building interactive applications. In FlashKraft, it enables:

- Clear separation of concerns
- Predictable state management
- Easy-to-test logic
- Type-safe async operations
- Maintainable codebase

By following these patterns, your application remains:
- **Scalable**: Easy to add features
- **Reliable**: Fewer bugs through type safety
- **Maintainable**: Clear structure and data flow
- **Testable**: Pure functions and explicit effects

## Further Reading

- [Iced Documentation](https://docs.rs/iced/)
- [The Elm Architecture Guide](https://guide.elm-lang.org/architecture/)
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [Command Pattern](https://en.wikipedia.org/wiki/Command_pattern)
