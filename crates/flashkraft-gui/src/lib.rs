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

// ── GUI entry point ───────────────────────────────────────────────────────────

/// Entry point for the Iced desktop GUI.
/// Called by the published `flashkraft` crate's GUI binary.
pub fn run_gui() -> iced::Result {
    use iced::{Settings, Task};

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--flash-helper") {
        let image_path = args.get(2).map(String::as_str).unwrap_or_else(|| {
            eprintln!("flash-helper: missing <image_path> argument");
            std::process::exit(2);
        });
        let device_path = args.get(3).map(String::as_str).unwrap_or_else(|| {
            eprintln!("flash-helper: missing <device_path> argument");
            std::process::exit(2);
        });
        flashkraft_core::flash_helper::run(image_path, device_path);
        std::process::exit(0);
    }

    iced::application(
        "FlashKraft - OS Image Writer",
        FlashKraft::update,
        FlashKraft::view,
    )
    .subscription(FlashKraft::subscription)
    .theme(|state: &FlashKraft| state.theme.clone())
    .settings(Settings {
        fonts: vec![iced_fonts::BOOTSTRAP_FONT_BYTES.into()],
        ..Default::default()
    })
    .window(iced::window::Settings {
        size: iced::Size::new(1300.0, 700.0),
        resizable: false,
        decorations: true,
        ..Default::default()
    })
    .run_with(|| {
        let initial_state = FlashKraft::new();
        let initial_command = Task::perform(
            flashkraft_core::commands::load_drives(),
            Message::DrivesRefreshed,
        );
        (initial_state, initial_command)
    })
}

// Re-export domain types from core so downstream code can do
// `use flashkraft_gui::{DriveInfo, ImageInfo}` without knowing about
// flashkraft-core directly.
pub use flashkraft_core::{DriveInfo, ImageInfo};
