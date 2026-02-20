//! FlashKraft TUI — Ratatui terminal application entry point
//!
//! ## Dual-mode binary
//!
//! This binary serves two distinct roles depending on its argv:
//!
//! | Invocation | Role |
//! |---|---|
//! | `flashkraft-tui` | Normal TUI startup |
//! | `pkexec flashkraft-tui --flash-helper <image> <device>` | Privileged flash helper |
//!
//! The `--flash-helper` branch is entered automatically by [`tui::flash_runner`]
//! via `pkexec`; it writes structured progress lines to stdout and exits.
//!
//! ## Module layout
//!
//! ```text
//! crate
//! ├── domain   ← re-exported from flashkraft_core::domain
//! ├── core     ← re-exported from flashkraft_core (commands, flash_writer, …)
//! └── tui      ← Ratatui front-end (app / events / flash_runner / ui)
//! ```
//!
//! The `crate::domain` and `crate::core` aliases let the `tui` submodules use
//! short paths (e.g. `crate::domain::DriveInfo`) without depending on the
//! external crate name directly.

// ── Core re-exports ───────────────────────────────────────────────────────────

/// Re-export `flashkraft_core` under the short alias `core` so that
/// `crate::core::commands::load_drives()`, `crate::core::flash_writer::*`,
/// etc. resolve correctly from every submodule.
pub mod core {
    pub use flashkraft_core::commands;
    pub use flashkraft_core::domain;
    pub use flashkraft_core::flash_helper;
    pub use flashkraft_core::flash_writer;
    pub use flashkraft_core::utils;
}

/// Re-export `flashkraft_core::domain` at the crate root so that
/// `crate::domain::DriveInfo` / `crate::domain::ImageInfo` resolve.
pub use flashkraft_core::domain;

// ── TUI submodules ────────────────────────────────────────────────────────────

pub mod tui;

// ── Std / external imports ────────────────────────────────────────────────────

use std::io;
use std::panic;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use tui::{app::App, events::handle_key, ui::render};

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // ── Privileged flash-helper mode ─────────────────────────────────────────
    //
    // When the application is re-launched under `pkexec` by the flash runner,
    // the first argument will be `--flash-helper`.  In that mode we skip the
    // TUI entirely, run the synchronous flash pipeline, and exit.
    {
        let args: Vec<String> = std::env::args().collect();
        if args.get(1).map(String::as_str) == Some("--flash-helper") {
            let image_path = match args.get(2) {
                Some(p) => p.as_str(),
                None => {
                    eprintln!("flash-helper: missing <image_path> argument");
                    std::process::exit(2);
                }
            };
            let device_path = match args.get(3) {
                Some(p) => p.as_str(),
                None => {
                    eprintln!("flash-helper: missing <device_path> argument");
                    std::process::exit(2);
                }
            };

            flashkraft_core::flash_helper::run(image_path, device_path);
            std::process::exit(0);
        }
    }

    // ── Normal TUI startup ────────────────────────────────────────────────────

    // Install a panic hook that restores the terminal before printing the
    // panic message — otherwise the terminal is left in raw / alternate-screen
    // mode and the panic output is invisible or garbled.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_hook(info);
    }));

    // Initialise raw-mode + alternate screen.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application.
    let run_result = run_app(&mut terminal).await;

    // Restore terminal unconditionally (even if the app returned Err).
    restore_terminal()?;
    terminal.show_cursor()?;

    run_result
}

// ── Application event loop ────────────────────────────────────────────────────

/// Drive the [`App`] state machine until `should_quit` is set.
///
/// Each iteration:
/// 1. Tick the internal counter (used for animations).
/// 2. Drain any pending async channel messages (drive detection / flash events).
/// 3. Render a single frame.
/// 4. Block for up to 100 ms waiting for a keyboard event.
async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    let mut app = App::new();

    loop {
        // ── Tick ─────────────────────────────────────────────────────────────
        app.tick_count = app.tick_count.wrapping_add(1);

        // ── Poll async channels ───────────────────────────────────────────────
        app.poll_drives();
        app.poll_flash();

        // ── Render ────────────────────────────────────────────────────────────
        terminal.draw(|frame| render(&mut app, frame))?;

        // ── Keyboard events (100 ms timeout) ──────────────────────────────────
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key);
            }
        }

        // ── Quit guard ────────────────────────────────────────────────────────
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

// ── Terminal cleanup helper ───────────────────────────────────────────────────

/// Disable raw mode and leave the alternate screen.
///
/// Called both on normal exit and from the panic hook so the terminal is
/// always left in a usable state.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
