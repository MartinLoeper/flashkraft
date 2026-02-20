//! FlashKraft Core
//!
//! This crate contains all shared, framework-free logic that is reused by
//! both the Iced desktop GUI (`flashkraft-gui`) and the Ratatui TUI
//! (`flashkraft-tui`).
//!
//! ## Contents
//!
//! | Module | What lives here |
//! |--------|-----------------|
//! | [`domain`] | Domain models — [`DriveInfo`], [`ImageInfo`], drive constraints |
//! | [`flash_helper`] | Privileged flash pipeline executed via `pkexec` |
//! | [`flash_writer`] | Wire-protocol parser for helper ↔ supervisor communication |
//! | [`commands`] | Async helpers — drive detection |
//! | [`utils`] | Debug-logging macros (`debug_log!`, `flash_debug!`, …) |
//!
//! ## Dependency policy
//!
//! This crate intentionally has **no** GUI or TUI dependencies (no `iced`,
//! no `ratatui`, no `crossterm`).  It may only depend on:
//! - OS / system crates (`sysinfo`, `nix`, `sha2`, …)
//! - Async utilities (`tokio`, `futures`, `futures-timer`)
//! - Persistence (`sled`, `dirs`)

// Utility macros must be declared first so they are available to every
// subsequent module via the implicit `#[macro_use]` on the crate root.
#[macro_use]
pub mod utils;

pub mod commands;
pub mod domain;
pub mod flash_helper;
pub mod flash_writer;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use domain::{DriveInfo, ImageInfo};
