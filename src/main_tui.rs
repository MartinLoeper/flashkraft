//! FlashKraft — TUI entry point
//!
//! This is the published `flashkraft` crate's TUI binary.
//! All logic lives in the `flashkraft-tui` workspace crate.
//!
//! Install with:
//!   cargo install flashkraft --features tui

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

        flashkraft_tui::run_flash_helper(image_path, device_path);
        std::process::exit(0);
    }

    flashkraft_tui::run().await
}
