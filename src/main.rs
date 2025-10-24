//! FlashKraft - OS Image Writer
//!
//! A Balena Etcher-inspired application built with Rust and Iced,
//! following The Elm Architecture pattern.
//!
//! # The Elm Architecture
//!
//! This application is structured around four core concepts:
//!
//! 1. **Model** (`model/`) - Application state
//! 2. **Message** (`message/`) - Events that trigger state changes
//! 3. **Update** (`update.rs`) - State transition logic
//! 4. **View** (`view/`) - UI rendering based on state
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

mod command;
mod flash_subscription;
mod icons;
mod message;
mod model;
mod storage;
mod update;
mod view;

use iced::{Element, Settings, Subscription, Task};

use flash_subscription::FlashProgress;
use message::Message;
use model::FlashKraft;

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
    .window_size((900.0, 600.0))
    .run_with(|| {
        let initial_state = FlashKraft::new();
        let initial_command = Task::perform(command::load_drives(), Message::DrivesRefreshed);
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
        // Optionally log messages for debugging
        #[cfg(debug_assertions)]
        println!("[DEBUG] Message: {:?}", message);

        update::update(self, message)
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
    /// This enables streaming progress updates from the flash operation.
    fn subscription(&self) -> Subscription<Message> {
        if self.flashing_active {
            if let (Some(image), Some(target)) = (&self.selected_image, &self.selected_target) {
                return flash_subscription::flash_progress(
                    image.path.clone(),
                    target.device_path.clone().into(),
                )
                .map(|progress| match progress {
                    FlashProgress::Progress(p) => Message::FlashProgressUpdate(p),
                    FlashProgress::Message(msg) => Message::FlashStatusMessage(msg),
                    FlashProgress::Completed => Message::FlashCompleted(Ok(())),
                    FlashProgress::Failed(err) => Message::FlashCompleted(Err(err)),
                });
            }
        }
        Subscription::none()
    }
}
