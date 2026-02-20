//! TUI Module — Ratatui-based terminal user interface for FlashKraft
//!
//! This module provides a full-featured terminal UI that mirrors the
//! functionality of the Iced GUI application, using:
//!
//! - [`ratatui`] for terminal rendering
//! - [`crossterm`] for terminal event handling
//! - [`tui_slider`] for flash progress display
//! - [`tui_piechart`] for drive storage visualisation
//!
//! # Architecture
//!
//! The TUI follows the same Elm-inspired architecture as the GUI:
//!
//! ```text
//! Event (keyboard/tick) → update(app, event) → render(app, frame)
//! ```
//!
//! All heavy work (drive detection, flash operation) is offloaded to
//! Tokio tasks; results arrive via [`tokio::sync::mpsc`] channels that
//! the main event loop polls on every tick.

pub mod app;
pub mod events;
pub mod flash_runner;
pub mod ui;
