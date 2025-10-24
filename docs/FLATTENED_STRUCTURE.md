# Flattened Project Structure

## Overview

The FlashKraft project has been refactored to use a **flattened module structure** that eliminates nested `mod.rs` files with mixed logic. This approach provides better clarity, easier navigation, and cleaner separation of concerns.

## Structure Philosophy

### Before: Nested Modules
```
src/
├── command/
│   └── mod.rs          # Command logic mixed with module declaration
├── message/
│   └── mod.rs          # Message logic mixed with module declaration
├── model/
│   ├── mod.rs          # Model logic + re-exports
│   ├── drive_info.rs   # Drive information
│   └── image_info.rs   # Image information
├── view/
│   ├── mod.rs          # View entry + delegation
│   └── components.rs   # Actual view components
├── update.rs           # Update logic
└── main.rs             # Application entry
```

**Problems with nested structure:**
- Multiple `mod.rs` files with mixed responsibilities
- Unclear where to find specific functionality
- Extra indirection through module re-exports
- More files to navigate when working with related code

### After: Flattened Modules
```
src/
├── command.rs          # All command/side-effect logic
├── message.rs          # All message/event definitions
├── model.rs            # Complete model: FlashKraft + DriveInfo + ImageInfo
├── view.rs             # Complete view: entry point + all components
├── update.rs           # Update/state transition logic
└── main.rs             # Application entry point
```

**Benefits of flattened structure:**
- One file per major concern (following The Elm Architecture)
- Clear, predictable file names
- Easy to find and navigate to specific functionality
- All related code in one place
- No `mod.rs` files with mixed logic

## File Organization

### `src/main.rs`
**Purpose:** Application entry point and Iced integration

**Contents:**
- Application struct implementing `iced::Application`
- Main function and initialization
- Minimal glue code between Iced and our Elm Architecture

### `src/model.rs`
**Purpose:** Complete state definition (The "Model" in Elm Architecture)

**Contents:**
- `FlashKraft` - Main application state struct
- `DriveInfo` - Drive information struct
- `ImageInfo` - Image file information struct
- All state-related helper methods
- All model tests

**Organization pattern:**
```rust
// ============================================================================
// DriveInfo - Information about a storage drive
// ============================================================================
[DriveInfo struct and implementation]

// ============================================================================
// ImageInfo - Information about a disk image file
// ============================================================================
[ImageInfo struct and implementation]

// ============================================================================
// FlashKraft - Main Application State
// ============================================================================
[FlashKraft struct and implementation]

// ============================================================================
// Tests
// ============================================================================
[All model-related tests]
```

### `src/message.rs`
**Purpose:** Event/message definitions (The "Message" in Elm Architecture)

**Contents:**
- `Message` enum with all possible application events
- User interaction messages
- Async result messages
- Comprehensive documentation for each variant

### `src/update.rs`
**Purpose:** State transitions (The "Update" in Elm Architecture)

**Contents:**
- `update()` function that processes messages
- Pure state transition logic
- Command generation for side effects
- Update function tests

### `src/view.rs`
**Purpose:** UI rendering (The "View" in Elm Architecture)

**Contents:**
- `view()` - Main entry point that routes to appropriate view
- Main application view components
- Device selection modal
- Progress/error/complete views
- All view helper functions
- View tests

**Organization pattern:**
```rust
// ============================================================================
// Main View Entry Point
// ============================================================================
[view() function and routing logic]

// ============================================================================
// Main Application View
// ============================================================================
[Normal application state UI]

// ============================================================================
// Selection Panels
// ============================================================================
[Image/target/flash selection panels]

// ============================================================================
// Device Selection Modal
// ============================================================================
[Table-based device selector]

// ============================================================================
// State-Specific Views
// ============================================================================
[Flashing/error/complete views]

// ============================================================================
// Tests
// ============================================================================
[View-related tests]
```

### `src/command.rs`
**Purpose:** Side effects and async operations (The "Command" in Elm Architecture)

**Contents:**
- `select_image_file()` - File picker dialog
- `load_drives()` - Drive detection
- `flash_image()` - Flashing operation
- All I/O and system interaction code
- Command tests

## The Elm Architecture Mapping

The flattened structure maps cleanly to The Elm Architecture:

| Elm Architecture | FlashKraft File | Responsibility |
|------------------|-----------------|----------------|
| **Model** | `model.rs` | Complete application state |
| **Message** | `message.rs` | All possible events |
| **Update** | `update.rs` | State transitions + side effects |
| **View** | `view.rs` | UI rendering |
| **Command** | `command.rs` | Async operations |

## Navigation Guide

### "Where do I find...?"

- **State definition?** → `model.rs`
- **Event types?** → `message.rs`
- **State transition logic?** → `update.rs`
- **UI components?** → `view.rs`
- **File dialogs / drive detection?** → `command.rs`
- **Application initialization?** → `main.rs`

### "Where do I add...?"

- **A new state field?** → Add to `FlashKraft` struct in `model.rs`
- **A new user action?** → Add message variant to `message.rs`, handle in `update.rs`
- **A new UI component?** → Add function to `view.rs` (with section comment)
- **A new async operation?** → Add function to `command.rs`
- **A new data type?** → Add to `model.rs` (with section comment)

## Code Organization Patterns

### Section Markers
Each file uses clear section markers to organize related functionality:

```rust
// ============================================================================
// Section Name
// ============================================================================
```

This makes it easy to:
- Quickly scan file contents
- Navigate using editor outline/folding
- Understand the logical organization
- Add new code in the right place

### Test Organization
All tests live in the same file as the code they test, under a clearly marked test section:

```rust
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    // ... tests here
}
```

## Benefits of This Structure

### For Development
- **Faster navigation:** Know exactly which file contains what you need
- **Less context switching:** Related code is together
- **Easier refactoring:** Clear boundaries between concerns
- **Better code review:** Changes grouped by architectural layer

### For Learning
- **Clear architecture:** File structure directly reflects Elm Architecture
- **Easy to explore:** Start at `main.rs`, follow the flow
- **Self-documenting:** File names and structure explain the design
- **Good example:** Clean pattern for other Iced/Elm projects

### For Maintenance
- **Predictable locations:** New team members quickly oriented
- **Minimal indirection:** No hunting through module re-exports
- **Clear ownership:** Each concern has exactly one home
- **Easy to extend:** Clear patterns for adding functionality

## Migration from Nested Structure

If you have an existing nested structure, flatten it by:

1. **Combine nested modules:** Merge `mod.rs` with related files
2. **Add section markers:** Organize combined code with `// ===` markers
3. **Update imports:** Change from `mod module; use module::Type;` to just module files
4. **Move tests:** Keep tests with the code they test
5. **Verify:** Run `cargo build` and `cargo test`

## Guidelines

### When to create a new file
- When adding a completely new architectural concern (rare in Elm Architecture)
- When a file becomes very large (>1000 lines) - split by clear logical sections
- When functionality is truly independent and reusable

### When NOT to create a new file
- For individual structs (group related types together)
- For "helper" functions (put them with the code that uses them)
- To mirror a directory structure from another project

### Keep it flat
- Prefer `thing.rs` over `thing/mod.rs`
- Group related functionality in sections within files
- Only introduce nesting if there's a clear, compelling reason

## Conclusion

The flattened structure makes FlashKraft easier to understand, navigate, and maintain. It directly reflects The Elm Architecture and provides a clean, predictable organization that scales well for projects of this size and complexity.

For a project following The Elm Architecture, you typically need just **5-6 core files** (model, message, update, view, command, main). Anything beyond that should have a very good reason.