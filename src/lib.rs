//! FlashKraft Library
//!
//! This module exposes the core FlashKraft components for use in examples
//! and external integrations.

// Utility modules must be declared first to make macros available
#[macro_use]
pub mod utils;

pub mod components;
pub mod core;
pub mod domain;
pub mod tui;
pub mod view;

// Re-export commonly used types
pub use core::{FlashKraft, Message};
pub use domain::{DriveInfo, ImageInfo};
