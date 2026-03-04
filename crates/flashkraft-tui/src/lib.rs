//! FlashKraft TUI — Library crate
//!
//! This crate exposes the full Ratatui terminal UI as a library so that:
//! - The `flashkraft-tui` binary can stay thin (argument parsing + `lib::run()`)
//! - Examples can import types directly from `flashkraft_tui::`
//!
//! ## Module layout
//!
//! ```text
//! flashkraft_tui
//! ├── domain  ← re-exported from flashkraft_core::domain
//! ├── core    ← re-exported from flashkraft_core (commands, flash_writer, …)
//! └── tui     ← Ratatui front-end (app / events / flash_runner / ui)
//! ```
//!
//! The file-browser widget is provided by the [`tui-file-explorer`](https://crates.io/crates/tui-file-explorer)
//! crate and consumed directly via `tui_file_explorer::*`.

// ── Core re-exports ───────────────────────────────────────────────────────────

/// Re-export `flashkraft_core` under the short alias `core` so that
/// `crate::core::commands::load_drives()`, `crate::core::flash_helper::*`,
/// etc. resolve correctly from every submodule and from examples via
/// `flashkraft_tui::core::*`.
pub mod core {
    pub use flashkraft_core::commands;
    pub use flashkraft_core::domain;
    pub use flashkraft_core::flash_helper;
    pub use flashkraft_core::utils;
}

/// Re-export `flashkraft_core::domain` at the crate root so that
/// `crate::domain::DriveInfo` / `crate::domain::ImageInfo` resolve in
/// submodules, and so that examples can write `flashkraft_tui::domain::*`.
pub use flashkraft_core::domain;

// ── TUI submodules ────────────────────────────────────────────────────────────

/// Ratatui front-end — app state, event handling, flash runner, UI rendering.
///
/// Submodules: `app` (state machine), `events` (key handling),
/// `flash_runner` (background flash task), `ui` (frame rendering).
pub mod tui;

// ── Convenience re-exports for examples and tests ────────────────────────────

pub use flashkraft_core::flash_helper;
pub use tui::app::{App, AppScreen, FlashEvent, InputMode, UsbEntry};
pub use tui::events::handle_key;
pub use tui::ui::render;
pub use tui_file_explorer::{ExplorerOutcome, FileExplorer, FsEntry};

// ── Public event-loop API ─────────────────────────────────────────────────────

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

/// Set up the terminal, run the application event loop, then restore the
/// terminal on exit (or on panic).
///
/// This is the single entry point called by `main.rs`.  Having it here lets
/// integration tests and examples exercise the loop without spawning a
/// subprocess.
pub async fn run() -> Result<()> {
    // Install a panic hook that restores the terminal before printing the
    // panic message — otherwise the output is invisible in raw / alt-screen
    // mode.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_hook(info);
    }));

    // Initialise raw mode + alternate screen.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Drive the application.
    let run_result = run_app(&mut terminal).await;

    // Restore unconditionally (even if the app returned Err).
    restore_terminal()?;
    terminal.show_cursor()?;

    run_result
}

/// Drive the [`App`] state machine until `should_quit` is set.
///
/// Each iteration:
/// 1. Tick the internal counter (used for animations).
/// 2. Drain any pending async channel messages (drive detection / flash / hotplug).
/// 3. Render a single frame.
/// 4. Block for up to 100 ms waiting for a keyboard event.
///
/// The generic backend parameter makes the function testable with ratatui's
/// `TestBackend` without touching real terminal infrastructure.
pub async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    let mut app = App::new();

    // ── USB hotplug watcher ───────────────────────────────────────────────────
    //
    // Spawn a background task that listens for USB connect / disconnect events
    // via `nusb::watch_devices()` and forwards a bare `()` trigger over an
    // unbounded channel.  `poll_hotplug()` drains the channel each tick and
    // starts a fresh drive enumeration when triggered.
    //
    // The task lives for the entire lifetime of the application.  If the OS
    // refuses to create the watch (no USB subsystem) we log and move on —
    // the manual r/F5 refresh path still works normally.
    {
        use flashkraft_core::commands::watch_usb_events;
        use futures::StreamExt as _;
        use tokio::sync::mpsc;

        let (tx, rx) = mpsc::unbounded_channel::<()>();
        app.hotplug_rx = Some(rx);

        tokio::spawn(async move {
            match watch_usb_events() {
                Ok(mut stream) => {
                    while stream.next().await.is_some() {
                        // A send failure means the App (and its receiver) was
                        // dropped — the TUI is shutting down; exit the task.
                        if tx.send(()).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[hotplug] watch_usb_events failed: {e}");
                }
            }
        });
    }

    loop {
        // ── Tick ─────────────────────────────────────────────────────────────
        app.tick_count = app.tick_count.wrapping_add(1);

        // ── Poll async channels ───────────────────────────────────────────────
        app.poll_hotplug();
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

/// Disable raw mode and leave the alternate screen.
///
/// Called both on normal exit and from the panic hook so the terminal is
/// always left in a usable state.
pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
