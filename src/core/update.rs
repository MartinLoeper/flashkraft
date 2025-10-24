//! Update Logic - The Elm Architecture
//!
//! This module contains the update function that processes messages
//! and updates the application state. This is the core of The Elm
//! Architecture where all state transitions occur.

use iced::Task;

use crate::core::commands;
use crate::core::message::Message;
use crate::core::state::FlashKraft;
use crate::domain::ImageInfo;

/// Update the application state based on a message
///
/// This is the heart of The Elm Architecture. It's a pure function that:
/// 1. Takes the current state and a message
/// 2. Updates the state
/// 3. Returns a Command for side effects (or Task::none())
///
/// # Arguments
///
/// * `state` - Mutable reference to the application state
/// * `message` - The message to process
///
/// # Returns
///
/// A Command that may trigger async operations, or Task::none()
pub fn update(state: &mut FlashKraft, message: Message) -> Task<Message> {
    match message {
        // ====================================================================
        // User Interaction Messages
        // ====================================================================
        Message::SelectImageClicked => {
            // Spawn async file selection dialog
            Task::perform(commands::select_image_file(), Message::ImageSelected)
        }

        Message::RefreshDrivesClicked => {
            // Spawn async drive detection
            Task::perform(commands::load_drives(), Message::DrivesRefreshed)
        }

        Message::TargetDriveClicked(drive) => {
            // Update selected target
            state.selected_target = Some(drive);
            state.error_message = None;
            // Close the device selection view
            state.device_selection_open = false;
            Task::none()
        }

        Message::OpenDeviceSelection => {
            // Open the device selection view
            state.device_selection_open = true;
            // Refresh drives when opening the view
            Task::perform(commands::load_drives(), Message::DrivesRefreshed)
        }

        Message::CloseDeviceSelection => {
            // Close the device selection view
            state.device_selection_open = false;
            Task::none()
        }

        Message::FlashClicked => {
            // Validate we can flash
            if state.is_ready_to_flash() {
                // Start flashing - activate subscription
                state.flash_progress = Some(0.0);
                state.error_message = None;
                state.flashing_active = true;

                Task::none()
            } else {
                // Set error if trying to flash without selections
                state.error_message =
                    Some("Please select both an image file and a target drive".to_string());
                Task::none()
            }
        }

        Message::ResetClicked => {
            // Reset all state
            state.reset();

            // Refresh drives list
            Task::perform(commands::load_drives(), Message::DrivesRefreshed)
        }

        Message::CancelClicked => {
            // Cancel current selections
            state.cancel_selections();
            Task::none()
        }

        Message::CancelFlash => {
            // Cancel ongoing flash operation
            state.flashing_active = false;
            state.flash_progress = None;
            state.flash_bytes_written = 0;
            state.flash_speed_mb_s = 0.0;
            state.error_message = Some("Flash operation cancelled by user".to_string());
            Task::none()
        }

        // ====================================================================
        // Async Result Messages
        // ====================================================================
        Message::ImageSelected(maybe_path) => {
            // Update selected image
            state.selected_image = maybe_path.map(ImageInfo::from_path);
            state.error_message = None;
            Task::none()
        }

        Message::DrivesRefreshed(drives) => {
            // Update available drives list
            state.available_drives = drives;
            Task::none()
        }

        Message::FlashProgressUpdate(progress, bytes, speed) => {
            // Update flash progress from subscription
            state.flash_progress = Some(progress);
            state.flash_bytes_written = bytes;
            state.flash_speed_mb_s = speed;
            // Update animated progress bar
            state.animated_progress.set_progress(progress);
            Task::none()
        }

        Message::AnimationTick => {
            // Tick animation for progress bar effects with speed-based scaling
            state.animated_progress.tick(state.flash_speed_mb_s);

            // Increment animation time for progress line glow effects
            // Scale based on transfer speed for dynamic animations
            let speed_multiplier = (state.flash_speed_mb_s / 20.0).clamp(0.3, 3.0);
            state.animation_time += 0.016 * speed_multiplier; // ~60 FPS baseline
            Task::none()
        }

        Message::Status(_message) => {
            // Log status message in debug builds
            #[cfg(debug_assertions)]
            println!("[STATUS] {}", _message);
            Task::none()
        }

        Message::FlashCompleted(result) => {
            // Deactivate subscription
            state.flashing_active = false;

            match result {
                Ok(()) => {
                    // Flash succeeded
                    state.flash_progress = Some(1.0);
                    state.error_message = None;
                }
                Err(error_message) => {
                    // Flash failed
                    state.flash_progress = None;
                    state.error_message = Some(error_message);
                }
            }
            Task::none()
        }

        Message::ThemeChanged(theme) => {
            // Update the application theme
            state.theme = theme.clone();

            // Update animated progress bar theme
            state.animated_progress.set_theme(theme.clone());

            // Save theme to persistent storage
            if let Some(storage) = &state.storage {
                if let Err(e) = storage.save_theme(&theme) {
                    eprintln!("Failed to save theme preference: {}", e);
                }
            }

            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DriveInfo;
    use std::path::PathBuf;

    #[test]
    fn test_cancel_clicked() {
        let mut state = FlashKraft::new();
        state.selected_image = Some(ImageInfo {
            path: PathBuf::from("/tmp/test.img"),
            name: "test.img".to_string(),
            size_mb: 100.0,
        });

        let _ = update(&mut state, Message::CancelClicked);

        assert!(state.selected_image.is_none());
    }

    #[test]
    fn test_target_drive_clicked() {
        let mut state = FlashKraft::new();
        let drive = DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );

        let _ = update(&mut state, Message::TargetDriveClicked(drive.clone()));

        assert_eq!(state.selected_target.as_ref().unwrap().name, "USB");
    }

    #[test]
    fn test_image_selected() {
        let mut state = FlashKraft::new();
        let path = PathBuf::from("/tmp/test.img");

        let _ = update(&mut state, Message::ImageSelected(Some(path.clone())));

        assert!(state.selected_image.is_some());
    }

    #[test]
    fn test_flash_clicked_without_selections() {
        let mut state = FlashKraft::new();

        let _ = update(&mut state, Message::FlashClicked);

        assert!(state.error_message.is_some());
        assert!(state.flash_progress.is_none());
    }

    #[test]
    fn test_flash_completed_success() {
        let mut state = FlashKraft::new();
        state.flash_progress = Some(0.5);

        let _ = update(&mut state, Message::FlashCompleted(Ok(())));

        assert_eq!(state.flash_progress, Some(1.0));
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_flash_completed_error() {
        let mut state = FlashKraft::new();
        state.flash_progress = Some(0.5);

        let _ = update(
            &mut state,
            Message::FlashCompleted(Err("Test error".to_string())),
        );

        assert!(state.flash_progress.is_none());
        assert_eq!(state.error_message.as_deref(), Some("Test error"));
    }

    #[test]
    fn test_reset_clicked() {
        let mut state = FlashKraft::new();
        state.selected_image = Some(ImageInfo {
            path: PathBuf::from("/tmp/test.img"),
            name: "test.img".to_string(),
            size_mb: 100.0,
        });
        state.flash_progress = Some(1.0);

        let _ = update(&mut state, Message::ResetClicked);

        assert!(state.selected_image.is_none());
        assert!(state.flash_progress.is_none());
    }
}
