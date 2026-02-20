//! FlashKraft TUI — Terminal User Interface entry point
//!
//! This binary provides a full-featured terminal UI that mirrors the
//! functionality of the Iced GUI, powered by:
//!
//! - [`ratatui`]      — terminal widget rendering
//! - [`crossterm`]    — terminal event handling & raw-mode management
//! - [`tui_slider`]   — flash-progress bar
//! - [`tui_piechart`] — drive storage & file-type pie charts
//! - [`tokio`]        — async runtime for drive detection and flash I/O
//!
//! # Flash-helper mode
//!
//! When invoked as:
//! ```text
//! flashkraft-tui --flash-helper <image_path> <device_path>
//! ```
//! the process runs the privileged flash pipeline and exits (identical to
//! the GUI binary's helper mode).  `pkexec` re-executes this binary with
//! elevated privileges; the TUI is never started in this mode.

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

// Pull in the library crate so we can reuse domain types, core commands,
// and the flash-helper entry point.
use flashkraft::tui::{app::App, events::handle_key, ui::render};

/// Tick interval — how often we redraw / poll channels when no key arrives.
const TICK_MS: u64 = 50;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Flash-helper mode ────────────────────────────────────────────────────
    // pkexec re-invokes this binary with `--flash-helper <image> <device>`.
    // We must handle that *before* touching any terminal / windowing code.
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

            flashkraft::core::flash_helper::run(image_path, device_path);
            std::process::exit(0);
        }
    }

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── Application state ─────────────────────────────────────────────────────
    let mut app = App::new();

    // ── Main event loop ───────────────────────────────────────────────────────
    let result = run_event_loop(&mut terminal, &mut app).await;

    // ── Terminal teardown ─────────────────────────────────────────────────────
    // Always restore the terminal even if we crashed.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("FlashKraft TUI error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

/// Run the ratatui event loop until the user requests a quit.
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    let tick_duration = Duration::from_millis(TICK_MS);

    loop {
        // ── Poll background channels ─────────────────────────────────────────
        app.poll_drives();
        app.poll_flash();
        app.tick_count = app.tick_count.wrapping_add(1);

        // ── Render frame ─────────────────────────────────────────────────────
        terminal.draw(|frame| render(app, frame))?;

        // ── Check quit flag ───────────────────────────────────────────────────
        if app.should_quit {
            break;
        }

        // ── Process terminal events (non-blocking poll) ───────────────────────
        if event::poll(tick_duration)? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(app, key);
                }
                // Resize events are handled automatically by ratatui on the
                // next draw call — no explicit action needed.
                Event::Resize(_, _) => {}
                // Mouse events are captured to suppress default terminal
                // selection behaviour, but we do not act on them.
                Event::Mouse(_) => {}
                _ => {}
            }
        }

        // ── Check quit flag again (key handler may have set it) ───────────────
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
