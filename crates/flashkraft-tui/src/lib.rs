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
//! ├── domain        ← re-exported from flashkraft_core::domain
//! ├── core          ← re-exported from flashkraft_core (commands, flash_writer, …)
//! ├── tui           ← Ratatui front-end (app / events / flash_runner / ui)
//! └── file_explorer ← self-contained file-browser widget
//! ```

// ── Core re-exports ───────────────────────────────────────────────────────────

/// Re-export `flashkraft_core` under the short alias `core` so that
/// `crate::core::commands::load_drives()`, `crate::core::flash_writer::*`,
/// etc. resolve correctly from every submodule and from examples via
/// `flashkraft_tui::core::*`.
pub mod core {
    pub use flashkraft_core::commands;
    pub use flashkraft_core::domain;
    pub use flashkraft_core::flash_helper;
    pub use flashkraft_core::flash_writer;
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

/// Self-contained file-explorer widget for Ratatui (the TUI framework).
///
/// This module has **no flashkraft-specific dependencies** — only `ratatui`,
/// `crossterm`, and the standard library — so it can be extracted and
/// published as a stand-alone crate without modification.
///
/// ## Quick start
///
/// ```no_run
/// use flashkraft_tui::file_explorer::{FileExplorer, ExplorerOutcome};
///
/// let mut explorer = FileExplorer::new(
///     dirs::home_dir().unwrap_or_default(),
///     vec!["iso".into(), "img".into()],
/// );
///
/// // In your render function (inside a ratatui draw closure):
/// // flashkraft_tui::file_explorer::render(&mut explorer, frame, frame.area());
///
/// // In your key handler:
/// # let key = crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
/// match explorer.handle_key(key) {
///     ExplorerOutcome::Selected(path) => { /* use path */ }
///     ExplorerOutcome::Dismissed      => { /* close explorer */ }
///     _                               => {}
/// }
/// ```
pub mod file_explorer;

// ── Convenience re-exports for examples and tests ────────────────────────────

pub use file_explorer::{ExplorerOutcome, FileExplorer, FsEntry};
pub use tui::app::{App, AppScreen, FlashEvent, InputMode, UsbEntry};
pub use tui::events::handle_key;
pub use tui::ui::render;

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
/// 2. Drain any pending async channel messages (drive detection / flash events).
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

/// Disable raw mode and leave the alternate screen.
///
/// Called both on normal exit and from the panic hook so the terminal is
/// always left in a usable state.
pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
