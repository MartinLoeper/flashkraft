//! Application State Module
//!
//! This module contains the main application state (FlashKraft struct)
//! which represents the complete state of the application at any point in time.

use crate::components::animated_progress::AnimatedProgress;
use crate::core::storage::Storage;
use crate::domain::{DriveInfo, ImageInfo};
use iced::Theme;

/// The main application state
///
/// This struct represents the complete state of the FlashKraft application.
/// All state is managed immutably and changes only through the `update` function.
#[derive(Debug)]
pub struct FlashKraft {
    /// Currently selected image file
    pub selected_image: Option<ImageInfo>,

    /// Currently selected target drive
    pub selected_target: Option<DriveInfo>,

    /// List of available drives detected on the system
    pub available_drives: Vec<DriveInfo>,

    /// Current flash progress (0.0 to 1.0), None if not flashing
    pub flash_progress: Option<f32>,

    /// Bytes written during flash operation
    pub flash_bytes_written: u64,

    /// Current transfer speed in MB/s
    pub flash_speed_mb_s: f32,

    /// Error message if an error occurred
    pub error_message: Option<String>,

    /// Whether the device selection view is currently open
    pub device_selection_open: bool,

    /// Whether a flash operation is currently active (for subscription)
    pub flashing_active: bool,

    /// Currently selected theme
    pub theme: Theme,

    /// Storage for persistent preferences
    pub storage: Option<Storage>,

    /// Animated progress bar for flash operations
    pub animated_progress: AnimatedProgress,

    /// Animation time for progress line glow effects (0.0 to infinity)
    pub animation_time: f32,
}

impl FlashKraft {
    /// Create a new FlashKraft instance with default values
    pub fn new() -> Self {
        // Try to initialize storage and load saved theme
        let storage = Storage::new().ok();
        let theme = storage
            .as_ref()
            .and_then(|s| s.load_theme())
            .unwrap_or(Theme::Dark);

        // Initialize animated progress with theme
        let mut animated_progress = AnimatedProgress::new();
        animated_progress.set_theme(theme.clone());

        Self {
            selected_image: None,
            selected_target: None,
            available_drives: Vec::new(),
            flash_progress: None,
            flash_bytes_written: 0,
            flash_speed_mb_s: 0.0,
            error_message: None,
            device_selection_open: false,
            flashing_active: false,
            theme,
            storage,
            animated_progress,
            animation_time: 0.0,
        }
    }

    /// Check if the application is ready to flash
    ///
    /// Returns true if both an image and target are selected
    pub fn is_ready_to_flash(&self) -> bool {
        self.selected_image.is_some() && self.selected_target.is_some()
    }

    /// Check if a flash operation is currently in progress
    pub fn is_flashing(&self) -> bool {
        self.flash_progress.is_some()
    }

    /// Check if the flash operation is complete
    pub fn is_flash_complete(&self) -> bool {
        matches!(self.flash_progress, Some(progress) if progress >= 1.0)
    }

    /// Check if there is an error
    pub fn has_error(&self) -> bool {
        self.error_message.is_some()
    }

    /// Reset the application state
    pub fn reset(&mut self) {
        self.selected_image = None;
        self.selected_target = None;
        self.flash_progress = None;
        self.flash_bytes_written = 0;
        self.flash_speed_mb_s = 0.0;
        self.error_message = None;
        self.device_selection_open = false;
        self.flashing_active = false;
    }

    /// Cancel current selections
    pub fn cancel_selections(&mut self) {
        self.selected_image = None;
        self.selected_target = None;
        self.flash_progress = None;
        self.flash_bytes_written = 0;
        self.flash_speed_mb_s = 0.0;
        self.error_message = None;
        self.device_selection_open = false;
        self.flashing_active = false;
    }
}

impl Default for FlashKraft {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_state() {
        let state = FlashKraft::new();
        assert!(state.selected_image.is_none());
        assert!(state.selected_target.is_none());
        assert!(state.available_drives.is_empty());
        assert!(!state.is_ready_to_flash());
        assert!(!state.device_selection_open);
    }

    #[test]
    fn test_is_ready_to_flash() {
        let mut state = FlashKraft::new();
        assert!(!state.is_ready_to_flash());

        state.selected_image = Some(ImageInfo {
            path: PathBuf::from("/tmp/test.img"),
            name: "test.img".to_string(),
            size_mb: 100.0,
        });
        assert!(!state.is_ready_to_flash());

        state.selected_target = Some(DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        ));
        assert!(state.is_ready_to_flash());
    }

    #[test]
    fn test_is_flashing() {
        let mut state = FlashKraft::new();
        assert!(!state.is_flashing());

        state.flash_progress = Some(0.5);
        assert!(state.is_flashing());
    }

    #[test]
    fn test_reset() {
        let mut state = FlashKraft::new();
        state.selected_image = Some(ImageInfo {
            path: PathBuf::from("/tmp/test.img"),
            name: "test.img".to_string(),
            size_mb: 100.0,
        });
        state.flash_progress = Some(0.5);
        state.error_message = Some("Error".to_string());
        state.device_selection_open = true;

        state.reset();

        assert!(state.selected_image.is_none());
        assert!(state.flash_progress.is_none());
        assert!(state.error_message.is_none());
        assert!(!state.device_selection_open);
    }
}
