//! FlashKraft GUI Library
//!
//! This crate contains the Iced desktop application for FlashKraft.
//!
//! ## Contents
//!
//! | Module | What lives here |
//! |--------|-----------------|
//! | [`core`] | Iced app state, messages, update logic, flash subscription, storage |
//! | [`components`] | Iced UI widgets and component renderers |
//! | [`view`] | Top-level view orchestration |
//! | [`utils`] | GUI-specific utilities (Bootstrap icon mapper) |
//!
//! ## Dependency on `flashkraft-core`
//!
//! All domain models, the flash pipeline, and drive-detection logic live in
//! the `flashkraft-core` crate.  This crate re-exports the most commonly
//! used types so callers only need to import from `flashkraft_gui`.

// GUI-specific utilities (Bootstrap icon mapper uses iced types)
#[macro_use]
pub mod utils;

pub mod components;
pub mod core;
pub mod view;

// ── Core re-exports ───────────────────────────────────────────────────────────

// Re-export `flashkraft_core::domain` at the crate root so that submodules can
// use `crate::domain::DriveInfo` / `crate::domain::ImageInfo` etc.
pub use flashkraft_core::domain;

// Re-export the `flash_debug!` macro from flashkraft_core so that
// `use crate::flash_debug;` in flash_subscription.rs resolves correctly.
pub use flashkraft_core::flash_debug;

// Re-export Iced app entry points
pub use core::{FlashKraft, Message};

// Re-export domain types from core so downstream code can do
// `use flashkraft_gui::{DriveInfo, ImageInfo}` without knowing about
// flashkraft-core directly.
pub use flashkraft_core::{DriveInfo, ImageInfo};
