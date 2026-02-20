//! FlashKraft TUI — binary entry point
//!
//! This file is intentionally thin.  All application logic lives in
//! `lib.rs` and the `tui/` submodules so that examples and integration
//! tests can import from `flashkraft_tui::` without also pulling in the
//! `main` function.
//!
//! ## Dual-mode binary
//!
//! | Invocation                                         | Role                      |
//! |----------------------------------------------------|---------------------------|
//! | `flashkraft-tui`                                   | Normal TUI startup        |
//! | `pkexec flashkraft-tui --flash-helper <img> <dev>` | Privileged flash helper   |
//!
//! The `--flash-helper` branch is entered automatically by
//! [`flashkraft_tui::tui::flash_runner`] via `pkexec`; it writes structured
//! progress lines to stdout and exits without touching any TUI code.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    flashkraft_tui::run().await
}
