//! FlashKraft TUI — Fully Functional Example
//!
//! Runs the complete Ratatui terminal application exactly as the
//! `flashkraft-tui` binary does.  The only difference is the banner
//! printed to stderr before the alternate screen opens.
//!
//! # What you get
//!
//! - Full keyboard-driven multi-screen UI
//! - Built-in file explorer  (Tab / Ctrl+F)
//! - Live drive detection    (r / F5)
//! - Drive info + pie-chart
//! - Confirm-flash checklist
//! - Real flash progress via pkexec (requires a connected USB target)
//!
//! # Keyboard quick-reference
//!
//! | Screen       | Key            | Action                        |
//! |--------------|----------------|-------------------------------|
//! | SelectImage  | i / Enter      | Start typing a path           |
//! | SelectImage  | Tab / Ctrl+F   | Open file browser             |
//! | SelectImage  | Esc / q        | Quit                          |
//! | BrowseImage  | j/k / ↑↓       | Navigate                      |
//! | BrowseImage  | Enter          | Descend / select file         |
//! | BrowseImage  | Backspace      | Ascend to parent              |
//! | BrowseImage  | Esc / q        | Dismiss                       |
//! | SelectDrive  | j/k / ↑↓       | Scroll drive list             |
//! | SelectDrive  | r / F5         | Refresh drives                |
//! | SelectDrive  | Enter / Space  | Confirm drive                 |
//! | SelectDrive  | Esc / b        | Go back                       |
//! | DriveInfo    | f / Enter      | Advance to confirm            |
//! | DriveInfo    | Esc / b        | Go back                       |
//! | ConfirmFlash | y / Y          | Begin flashing                |
//! | ConfirmFlash | n / Esc / b    | Go back                       |
//! | Flashing     | c / Esc        | Cancel                        |
//! | Complete     | r / R          | Reset to start                |
//! | Complete     | q / Esc        | Quit                          |
//! | Error        | r / Enter      | Reset to start                |
//! | Error        | q / Esc        | Quit                          |
//! | Any          | Ctrl+C / Ctrl+Q| Force quit                    |
//!
//! # Run
//!
//! ```bash
//! cargo run -p flashkraft-tui --example tui
//! # or via just:
//! just example-tui
//! ```

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Capture real UID for setuid-root privilege model ─────────────────────
    //
    // If the binary is installed setuid-root, getuid() returns the invoking
    // user's UID while geteuid() returns 0.  We store the real UID so the
    // flash pipeline can drop back to it after opening the block device.
    #[cfg(unix)]
    {
        let real_uid = nix::unistd::getuid();
        flashkraft_core::flash_helper::set_real_uid(real_uid.as_raw());
    }

    // ── Banner (written before alternate screen opens) ────────────────────────
    eprintln!();
    eprintln!("  ⚡ FlashKraft TUI");
    eprintln!("  ─────────────────────────────────────────────");
    eprintln!("  A keyboard-driven OS image writer.");
    eprintln!();
    eprintln!("  Quick start:");
    eprintln!("    1. Press i or Enter to type an image path");
    eprintln!("       (or Tab / Ctrl+F to browse for a file)");
    eprintln!("    2. Select a target drive with j / k, then Enter");
    eprintln!("    3. Review drive info, press f to continue");
    eprintln!("    4. Read the safety checklist, press y to flash");
    eprintln!("    5. Press q or Esc when done");
    eprintln!();
    eprintln!("  Requires pkexec (polkit) for the actual flash step.");
    eprintln!("  ─────────────────────────────────────────────");
    eprintln!();

    // ── Run the full TUI event loop ───────────────────────────────────────────
    flashkraft_tui::run().await
}
