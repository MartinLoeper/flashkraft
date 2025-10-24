//! Core Module
//!
//! This module contains the core application logic following The Elm Architecture:
//! - State management
//! - Message definitions
//! - Update logic
//! - Commands (side effects)
//! - Storage/persistence

pub mod commands;
pub mod flash_subscription;
pub mod message;
pub mod state;
pub mod storage;
pub mod update;

// Re-export for convenience
pub use message::Message;
pub use state::FlashKraft;
pub use update::update;
