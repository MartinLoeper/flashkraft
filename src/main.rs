//! FlashKraft - OS Image Writer
//!
//! A Balena Etcher-inspired application built with Rust and Iced,
//! following The Elm Architecture pattern.
//!
//! # The Elm Architecture
//!
//! This application is structured around four core concepts:
//!
//! 1. **Model** - Application state (`core/state.rs`)
//! 2. **Message** - Events that trigger state changes (`core/message.rs`)
//! 3. **Update** - State transition logic (`core/update.rs`)
//! 4. **View** - UI rendering based on state (`view.rs` + `components/`)
//!
//! ## Data Flow
//!
//! ```text
//! User Action → Message → Update → New State → View → UI
//!                           ↓
//!                        Command
//!                           ↓
//!                    Async Task → Message
//! ```
//!
//! ## Module Structure
//!
//! - `core/` - Core application logic (Elm Architecture)
//!   - `state.rs` - Application state (Model) + Elm methods
//!   - `message.rs` - Message definitions
//!   - `update.rs` - Update logic
//!   - `commands/` - Async commands (side effects)
//!   - `storage.rs` - Persistent storage
//!   - `flash_subscription.rs` - Flash operation subscription
//!
//! - `domain/` - Domain models
//!   - `drive_info.rs` - Drive information
//!   - `image_info.rs` - Image information
//!
//! - `components/` - UI components
//!   - `header.rs` - App header
//!   - `step_indicators.rs` - Step indicators
//!   - `progress_line.rs` - Animated progress lines
//!   - `selection_panels.rs` - Selection buttons
//!   - `device_selector.rs` - Device selection overlay
//!   - `status_views.rs` - Status views (flashing, error, complete)
//!   - `theme_selector.rs` - Theme selector
//!   - `animated_progress.rs` - Animated progress bar
//!
//! - `utils/` - Utility modules
//!   - `icons_bootstrap_mapper.rs` - Icon utilities
//!   - `logger.rs` - Logging macros
//!
//! - `view.rs` - Main view orchestration

// Utility modules must be declared first to make macros available
#[macro_use]
mod utils;

mod components;
mod core;
mod domain;
mod view;

use iced::{Settings, Task};

use core::{FlashKraft, Message};

/// Application entry point
///
/// Sets up and runs the Iced application with The Elm Architecture.
fn main() -> iced::Result {
    iced::application(
        "FlashKraft - OS Image Writer",
        FlashKraft::update,
        FlashKraft::view,
    )
    .subscription(FlashKraft::subscription)
    .theme(|state: &FlashKraft| state.theme.clone())
    .settings(Settings {
        fonts: vec![iced_fonts::BOOTSTRAP_FONT_BYTES.into()],
        ..Default::default()
    })
    .window(iced::window::Settings {
        size: iced::Size::new(1300.0, 700.0),
        resizable: false,
        decorations: true,
        ..Default::default()
    })
    .run_with(|| {
        // Initialize application state
        let initial_state = FlashKraft::new();

        // Load drives on startup
        let initial_command =
            Task::perform(core::commands::load_drives(), Message::DrivesRefreshed);

        (initial_state, initial_command)
    })
}
