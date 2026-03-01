//! FlashKraft TUI — binary entry point
//!
//! This file is intentionally thin.  All application logic lives in
//! `lib.rs` and the `tui/` submodules so that examples and integration
//! tests can import from `flashkraft_tui::` without also pulling in the
//! `main` function.
//!
//! ## Privilege model
//!
//! The installed binary carries the **setuid-root** bit:
//!
//! ```text
//! sudo chown root:root /usr/bin/flashkraft-tui
//! sudo chmod u+s       /usr/bin/flashkraft-tui
//! ```
//!
//! At startup we capture the **real** (unprivileged) UID via
//! `nix::unistd::getuid()` and store it via
//! `flashkraft_core::flash_helper::set_real_uid`.  The flash pipeline then
//! uses it to drop back to the real user immediately after opening the block
//! device file descriptor.
//!
//! No child process is spawned.  No pkexec.  No polkit policy file needed.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Capture the real UID before anything else ────────────────────────────
    //
    // If the binary is setuid-root, `getuid()` still returns the UID of the
    // user who launched it (the *real* UID), while `geteuid()` returns 0.
    // We save the real UID now so the flash pipeline can drop privileges back
    // to the invoking user after opening the block device.
    //
    // On non-Unix platforms this block compiles away.
    #[cfg(unix)]
    {
        let real_uid = nix::unistd::getuid();
        flashkraft_core::flash_helper::set_real_uid(real_uid.as_raw());
    }

    // ── Normal TUI startup ────────────────────────────────────────────────────
    flashkraft_tui::run().await
}
