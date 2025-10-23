//! Message - Events in The Elm Architecture
//!
//! This module defines all possible messages (events) that can occur
//! in the FlashKraft application. Messages are the only way to trigger
//! state changes, making the application predictable and debuggable.

use std::path::PathBuf;

use crate::model::DriveInfo;

/// All possible messages in the application
///
/// Messages represent events that can occur, either from user interactions
/// or as results of asynchronous operations (Commands).
#[derive(Debug, Clone)]
pub enum Message {
    // ========================================================================
    // User Interaction Messages
    // ========================================================================
    /// User clicked the "Select Image" button
    SelectImageClicked,

    /// User clicked the "Refresh Drives" button
    RefreshDrivesClicked,

    /// User clicked on a specific target drive
    TargetDriveClicked(DriveInfo),

    /// User clicked to open the device selection view
    OpenDeviceSelection,

    /// User clicked to close the device selection view
    CloseDeviceSelection,

    /// User clicked the "Flash" button
    FlashClicked,

    /// User clicked the "Reset" button (start over)
    ResetClicked,

    /// User clicked the "Cancel" button
    CancelClicked,

    // ========================================================================
    // Async Result Messages
    // ========================================================================
    /// Result from async image file selection
    ///
    /// Contains `Some(path)` if user selected a file, `None` if cancelled
    ImageSelected(Option<PathBuf>),

    /// Result from async drive detection
    ///
    /// Contains a list of detected drives
    DrivesRefreshed(Vec<DriveInfo>),

    /// Progress update from flash subscription
    ///
    /// Contains progress as a float between 0.0 and 1.0
    FlashProgressUpdate(f32),

    /// Status message from flash operation
    FlashStatusMessage(String),

    /// Result from async flash operation
    ///
    /// Contains `Ok(())` on success or `Err(message)` on failure
    FlashCompleted(Result<(), String>),
}
