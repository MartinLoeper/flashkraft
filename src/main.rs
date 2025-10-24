//! FlashKraft - OS Image Writer
//!
//! A Balena Etcher-inspired application built with Rust and Iced,
//! following The Elm Architecture pattern.
//!
//! # The Elm Architecture
//!
//! This application is structured around four core concepts:
//!
//! 1. **State** (`core/state.rs`) - Application state
//! 2. **Message** (`core/message.rs`) - Events that trigger state changes
//! 3. **Update** (`core/update.rs`) - State transition logic
//! 4. **View** (`view.rs` + `components/`) - UI rendering based on state
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
//!   - `state.rs` - Application state
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

use iced::{Element, Settings, Subscription, Task};

use core::flash_subscription::FlashProgress;
use core::{FlashKraft, Message};

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
        let initial_state = FlashKraft::new();
        let initial_command =
            Task::perform(core::commands::load_drives(), Message::DrivesRefreshed);
        (initial_state, initial_command)
    })
}

// ============================================================================
// Application Implementation - The Elm Architecture
// ============================================================================

impl FlashKraft {
    /// Update the application state based on a message
    ///
    /// This is the core of The Elm Architecture. All state changes
    /// flow through this function.
    fn update(&mut self, message: Message) -> Task<Message> {
        // Optionally log messages for debugging (exclude AnimationTick to avoid spam)
        if !matches!(message, Message::AnimationTick) {
            #[cfg(debug_assertions)]
            println!("[DEBUG] Message: {:?}", message);
        }

        core::update(self, message)
    }

    /// Render the user interface
    ///
    /// This is a pure function that describes what the UI should look
    /// like based on the current state.
    fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }

    /// Subscribe to long-running operations
    ///
    /// This enables streaming progress updates from the flash operation
    /// and animation ticks for the progress bar.
    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        // Flash progress subscription
        if self.flashing_active {
            if let (Some(image), Some(target)) = (&self.selected_image, &self.selected_target) {
                let flash_sub = core::flash_subscription::flash_progress(
                    image.path.clone(),
                    target.device_path.clone().into(),
                )
                .map(|progress| match progress {
                    FlashProgress::Progress(p, bytes, speed) => {
                        Message::FlashProgressUpdate(p, bytes, speed)
                    }
                    FlashProgress::Message(msg) => Message::FlashStatusMessage(msg),
                    FlashProgress::Completed => Message::FlashCompleted(Ok(())),
                    FlashProgress::Failed(err) => Message::FlashCompleted(Err(err)),
                });
                subscriptions.push(flash_sub);
            }

            // Animation tick subscription (during flash)
            let animation_sub = iced::window::frames().map(|_| Message::AnimationTick);
            subscriptions.push(animation_sub);
        } else {
            // Always run animation tick for progress line glow effects
            let animation_sub = iced::window::frames().map(|_| Message::AnimationTick);
            subscriptions.push(animation_sub);
        }

        Subscription::batch(subscriptions)
    }
}
