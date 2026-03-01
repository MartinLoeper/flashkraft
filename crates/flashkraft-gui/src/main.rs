//! FlashKraft GUI — application entry point
//!
//! ## Privilege model
//!
//! The installed binary carries the **setuid-root** bit:
//!
//! ```text
//! sudo chown root:root /usr/bin/flashkraft
//! sudo chmod u+s       /usr/bin/flashkraft
//! ```
//!
//! At startup we capture the **real** (unprivileged) UID via
//! `nix::unistd::getuid()` — before any `seteuid` call — and store it in
//! `flashkraft_core::flash_helper::set_real_uid`.  The flash pipeline then
//! uses it to drop back to the real user immediately after opening the block
//! device file descriptor.
//!
//! No child process is spawned.  No pkexec.  No polkit policy file needed.

use flashkraft_gui::{FlashKraft, Message};
use iced::{Settings, Task};

fn main() -> iced::Result {
    // ── Capture the real UID before anything else ────────────────────────────
    //
    // If the binary is setuid-root, `getuid()` still returns the UID of the
    // user who launched it (the *real* UID), while `geteuid()` returns 0.
    // We save the real UID now so the flash pipeline can drop privileges back
    // to the invoking user after opening the block device.
    //
    // On non-Unix platforms (e.g. Windows CI) this block compiles away.
    #[cfg(unix)]
    {
        let real_uid = nix::unistd::getuid();
        flashkraft_core::flash_helper::set_real_uid(real_uid.as_raw());
    }

    // ── Start the Iced GUI ───────────────────────────────────────────────────
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

        // Kick off drive detection immediately on startup.
        let initial_command = Task::perform(
            flashkraft_core::commands::load_drives(),
            Message::DrivesRefreshed,
        );

        (initial_state, initial_command)
    })
}
