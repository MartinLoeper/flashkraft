//! Domain Module
//!
//! This module contains the domain models for the FlashKraft application.
//! Domain models represent the core business entities and logic.

pub mod drive_info;
pub mod image_info;

// Re-export for convenience
pub use drive_info::DriveInfo;
pub use image_info::ImageInfo;
