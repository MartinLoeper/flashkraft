//! Domain Module
//!
//! This module contains the domain models for the FlashKraft application.
//! Domain models represent the core business entities and logic.

pub mod constraints;
pub mod drive_info;
pub mod image_info;

// Re-export for convenience
pub use constraints::{
    get_drive_image_compatibility_statuses, is_drive_valid, is_source_drive, is_system_drive,
    mark_invalid_drives, CompatibilityStatus, CompatibilityStatusType, LARGE_DRIVE_SIZE,
};
pub use drive_info::DriveInfo;
pub use image_info::ImageInfo;
