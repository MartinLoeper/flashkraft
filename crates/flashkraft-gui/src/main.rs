//! FlashKraft GUI — Iced desktop application entry point
//!
//! ## Dual-mode binary
//!
//! This binary serves two distinct roles depending on its argv:
//!
//! | Invocation | Role |
//! |---|---|
//! | `flashkraft` | Normal Iced GUI startup |
//! | `pkexec flashkraft --flash-helper <image> <device>` | Privileged flash helper |
//!
//! The `--flash-helper` branch is entered automatically by the flash
//! subscription via `pkexec`; it writes structured progress lines to stdout
//! and exits without ever touching any windowing code.

use flashkraft_gui::{FlashKraft, Message};
use iced::{Settings, Task};

/// Application entry point.
///
/// If the first argument is `--flash-helper`, we are running as a privileged
/// child process (launched via `pkexec`).  In that mode we perform the flash
/// operation and exit — the GUI is never started.
///
/// Normal invocation (no special arguments) starts the Iced GUI.
fn main() -> iced::Result {
    // ── Privileged helper mode ───────────────────────────────────────────────
    // Detect `--flash-helper <image_path> <device_path>` before touching any
    // GUI / windowing code.  pkexec re-executes this binary with root
    // privileges; we use the extra arguments to know which mode to run in.
    {
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

            // run() writes structured lines to stdout and calls
            // std::process::exit internally on failure.
            flashkraft_core::flash_helper::run(image_path, device_path);
            std::process::exit(0);
        }
    }

    // ── Normal GUI startup ───────────────────────────────────────────────────
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
        // Initialise application state.
        let initial_state = FlashKraft::new();

        // Load drives on startup.
        let initial_command = Task::perform(
            flashkraft_core::commands::load_drives(),
            Message::DrivesRefreshed,
        );

        (initial_state, initial_command)
    })
}
