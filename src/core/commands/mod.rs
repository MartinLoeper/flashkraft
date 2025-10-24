//! Commands Module
//!
//! This module contains all async command functions that perform side effects.
//! Commands are called from the update function and their results are converted
//! back into Messages, keeping the update function pure.

pub mod drive_detection;
pub mod file_selection;

// Re-export for convenience
pub use drive_detection::load_drives;
pub use file_selection::select_image_file;
