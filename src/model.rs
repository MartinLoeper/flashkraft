//! Model (State) - The Elm Architecture
//!
//! This module contains all the data structures that represent
//! the application state. Following the Elm Architecture, the
//! model is immutable and only changes through the update function.

use iced::Theme;
use std::path::PathBuf;

// ============================================================================
// DriveInfo - Information about a storage drive
// ============================================================================

/// Information about a storage drive
#[derive(Debug, Clone)]
pub struct DriveInfo {
    /// Name of the drive
    pub name: String,
    /// Mount point of the drive
    pub mount_point: String,
    /// Size of the drive in gigabytes
    pub size_gb: f64,
    /// Raw device path (e.g., /dev/sde)
    pub device_path: String,
}

impl DriveInfo {
    /// Create a new DriveInfo instance
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the drive
    /// * `mount_point` - Where the drive is mounted
    /// * `size_gb` - Size in gigabytes
    /// * `device_path` - Raw device path (e.g., /dev/sde)
    pub fn new(name: String, mount_point: String, size_gb: f64, device_path: String) -> Self {
        Self {
            name,
            mount_point,
            size_gb,
            device_path,
        }
    }

    /// Get a display string for the drive
    ///
    /// # Returns
    ///
    /// A formatted string showing name, size, and mount point
    #[allow(dead_code)]
    pub fn display_string(&self) -> String {
        format!(
            "{} - {:.2} GB ({})",
            self.name, self.size_gb, self.mount_point
        )
    }
}

impl PartialEq for DriveInfo {
    fn eq(&self, other: &Self) -> bool {
        self.device_path == other.device_path
    }
}

// ============================================================================
// ImageInfo - Information about a disk image file
// ============================================================================

/// Information about a disk image file
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// Full path to the image file
    pub path: PathBuf,
    /// Display name of the file
    pub name: String,
    /// Size of the file in megabytes
    pub size_mb: f64,
}

impl ImageInfo {
    /// Create a new ImageInfo from a PathBuf
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the image file
    ///
    /// # Returns
    ///
    /// An ImageInfo instance with extracted file name and size
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let size_mb = path
            .metadata()
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);

        Self {
            path,
            name,
            size_mb,
        }
    }
}

// ============================================================================
// FlashKraft - Main Application State
// ============================================================================

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

    /// Error message if an error occurred
    pub error_message: Option<String>,

    /// Whether the device selection view is currently open
    pub device_selection_open: bool,

    /// Whether a flash operation is currently active (for subscription)
    pub flashing_active: bool,

    /// Currently selected theme
    pub theme: Theme,
}

impl FlashKraft {
    /// Create a new FlashKraft instance with default values
    pub fn new() -> Self {
        Self {
            selected_image: None,
            selected_target: None,
            available_drives: Vec::new(),
            flash_progress: None,
            error_message: None,
            device_selection_open: false,
            flashing_active: false,
            theme: Theme::Dark,
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
        self.error_message = None;
        self.device_selection_open = false;
        self.flashing_active = false;
    }

    /// Cancel current selections
    pub fn cancel_selections(&mut self) {
        self.selected_image = None;
        self.selected_target = None;
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // DriveInfo tests
    #[test]
    fn test_drive_display_string() {
        let drive = DriveInfo::new(
            "USB Drive".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        assert_eq!(drive.display_string(), "USB Drive - 32.00 GB (/media/usb)");
    }

    #[test]
    fn test_drive_equality() {
        let drive1 = DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        let drive2 = DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        let drive3 = DriveInfo::new(
            "USB2".to_string(),
            "/media/usb2".to_string(),
            32.0,
            "/dev/sdc".to_string(),
        );

        assert_eq!(drive1, drive2);
        assert_ne!(drive1, drive3);
    }

    // FlashKraft state tests
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
