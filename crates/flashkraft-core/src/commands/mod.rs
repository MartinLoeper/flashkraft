//! Commands Module — Core Async Side Effects
//!
//! Contains async helper functions that perform OS-level side effects.
//! These are shared between both the GUI and TUI frontends.
//!
//! ## What belongs here
//!
//! - Drive / block-device detection ([`drive_detection`])
//!
//! ## What does NOT belong here
//!
//! - File-picker dialogs (`rfd`) — those are GUI-only and live in
//!   `flashkraft-gui::core::commands::file_selection`.

pub mod drive_detection;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use drive_detection::load_drives;
