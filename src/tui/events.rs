//! TUI Event Handling
//!
//! Maps crossterm [`KeyEvent`]s to [`App`] state transitions.
//! Every key press is routed through [`handle_key`], which delegates
//! to a screen-specific handler keeping each case small and focused.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, AppScreen, InputMode};

/// Process a single key event and mutate `app` accordingly.
///
/// Returns `true` if the event was consumed (no further handling needed).
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // ── Global shortcuts (work on every screen) ──────────────────────────────
    match key.code {
        // Ctrl-C / Ctrl-Q → quit (or cancel flash if one is running)
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.screen == AppScreen::Flashing {
                app.cancel_flash();
            } else {
                app.should_quit = true;
            }
            return true;
        }
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return true;
        }
        _ => {}
    }

    // ── Screen-specific handling ─────────────────────────────────────────────
    match app.screen {
        AppScreen::SelectImage => handle_select_image(app, key),
        AppScreen::SelectDrive => handle_select_drive(app, key),
        AppScreen::DriveInfo => handle_drive_info(app, key),
        AppScreen::ConfirmFlash => handle_confirm_flash(app, key),
        AppScreen::Flashing => handle_flashing(app, key),
        AppScreen::Complete => handle_complete(app, key),
        AppScreen::Error => handle_error(app, key),
    }
}

// ---------------------------------------------------------------------------
// Screen: SelectImage
// ---------------------------------------------------------------------------

fn handle_select_image(app: &mut App, key: KeyEvent) -> bool {
    match app.input_mode {
        InputMode::Editing => match key.code {
            // Confirm path.
            KeyCode::Enter => {
                match app.confirm_image() {
                    Ok(()) => {
                        // Kick off async drive detection.
                        start_drive_detection(app);
                    }
                    Err(msg) => {
                        app.error_message = msg;
                        app.screen = AppScreen::Error;
                    }
                }
                true
            }

            // Text editing.
            KeyCode::Char(c) => {
                app.image_insert(c);
                true
            }
            KeyCode::Backspace => {
                app.image_backspace();
                true
            }
            KeyCode::Delete => {
                // Delete character under cursor (shift cursor left first then
                // delete — simplest approach compatible with our helpers).
                app.image_cursor_right();
                app.image_backspace();
                true
            }
            KeyCode::Left => {
                app.image_cursor_left();
                true
            }
            KeyCode::Right => {
                app.image_cursor_right();
                true
            }
            KeyCode::Home => {
                app.image_cursor = 0;
                true
            }
            KeyCode::End => {
                app.image_cursor = app.image_input.chars().count();
                true
            }

            // Escape exits the app from the first screen.
            KeyCode::Esc => {
                app.should_quit = true;
                true
            }

            _ => false,
        },

        InputMode::Normal => match key.code {
            KeyCode::Enter | KeyCode::Char('i') => {
                app.input_mode = InputMode::Editing;
                true
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.should_quit = true;
                true
            }
            _ => false,
        },
    }
}

// ---------------------------------------------------------------------------
// Screen: SelectDrive
// ---------------------------------------------------------------------------

fn handle_select_drive(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Navigate list.
        KeyCode::Up | KeyCode::Char('k') => {
            app.drive_up();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.drive_down();
            true
        }

        // Confirm selection.
        KeyCode::Enter | KeyCode::Char(' ') => {
            if app.available_drives.is_empty() {
                return true; // nothing to select
            }
            match app.confirm_drive() {
                Ok(()) => {}
                Err(msg) => {
                    app.error_message = msg;
                    app.screen = AppScreen::Error;
                }
            }
            true
        }

        // Refresh drive list.
        KeyCode::Char('r') | KeyCode::F(5) => {
            start_drive_detection(app);
            true
        }

        // Go back to image selection.
        KeyCode::Backspace | KeyCode::Esc | KeyCode::Char('b') => {
            app.go_back();
            true
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Screen: DriveInfo
// ---------------------------------------------------------------------------

fn handle_drive_info(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Advance to confirmation.
        KeyCode::Enter | KeyCode::Char('f') => {
            app.advance_to_confirm();
            true
        }

        // Go back to drive selection.
        KeyCode::Backspace | KeyCode::Esc | KeyCode::Char('b') => {
            app.go_back();
            true
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Screen: ConfirmFlash
// ---------------------------------------------------------------------------

fn handle_confirm_flash(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Confirm and start flash.
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            match app.begin_flash() {
                Ok(()) => {}
                Err(msg) => {
                    app.error_message = msg;
                    app.screen = AppScreen::Error;
                }
            }
            true
        }

        // Cancel / go back.
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('b') => {
            app.go_back();
            true
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Screen: Flashing
// ---------------------------------------------------------------------------

fn handle_flashing(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Cancel the running flash operation.
        KeyCode::Char('c') | KeyCode::Esc => {
            app.cancel_flash();
            true
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Screen: Complete
// ---------------------------------------------------------------------------

fn handle_complete(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Scroll USB contents.
        KeyCode::Up | KeyCode::Char('k') => {
            app.contents_up();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.contents_down();
            true
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.contents_up();
            }
            true
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.contents_down();
            }
            true
        }

        // Flash another image — full reset.
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.reset();
            true
        }

        // Quit.
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            true
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Screen: Error
// ---------------------------------------------------------------------------

fn handle_error(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Retry — full reset.
        KeyCode::Char('r') | KeyCode::Enter => {
            app.reset();
            true
        }

        // Quit.
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            true
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Helper: kick off async drive detection
// ---------------------------------------------------------------------------

/// Spawn a Tokio task that calls [`crate::core::commands::load_drives`] and
/// sends the result through an [`tokio::sync::mpsc`] channel stored in `app`.
fn start_drive_detection(app: &mut App) {
    use tokio::sync::mpsc;

    app.drives_loading = true;
    app.available_drives.clear();
    app.drive_cursor = 0;

    let (tx, rx) = mpsc::unbounded_channel::<Vec<crate::domain::DriveInfo>>();
    app.drives_rx = Some(rx);

    tokio::spawn(async move {
        let drives = crate::core::commands::load_drives().await;
        let _ = tx.send(drives);
    });
}
