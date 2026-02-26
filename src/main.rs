//! FlashKraft — GUI entry point
//!
//! This is the published `flashkraft` crate's GUI binary.
//! All logic lives in the `flashkraft-gui` workspace crate.
//!
//! Install with:
//!   cargo install flashkraft --features gui

fn main() {
    if let Err(e) = flashkraft_gui::run_gui() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
