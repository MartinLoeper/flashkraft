//! Basic Usage Example - FlashKraft
//!
//! This example demonstrates how to run the FlashKraft application.
//! It shows the complete OS image writer with all features:
//! - Image file selection
//! - Device detection and selection
//! - Flash progress tracking
//! - Theme support
//! - Error handling
//!
//! Run with: cargo run --example basic_usage

use flashkraft::{FlashKraft, Message};
use iced::{Settings, Task};

fn main() -> iced::Result {
    println!("===========================================");
    println!("   FlashKraft - Basic Usage Example");
    println!("===========================================");
    println!();
    println!("This example runs the complete FlashKraft application.");
    println!();
    println!("Features demonstrated:");
    println!("  • Image file selection (ISO, IMG, DMG, ZIP)");
    println!("  • Automatic drive detection");
    println!("  • Device selection interface");
    println!("  • Flash progress tracking");
    println!("  • 21 beautiful themes");
    println!("  • Safe drive selection");
    println!();
    println!("Usage:");
    println!("  1. Click 'Select Image' to choose an OS image file");
    println!("  2. Click 'Select Target' to choose a USB/SD drive");
    println!("  3. Click 'Flash!' to start the flashing process");
    println!("  4. Click the theme button (🎨) to change themes");
    println!();
    println!("===========================================");
    println!();

    iced::application(
        "FlashKraft - Basic Usage Example",
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
        // Initialize application state
        let initial_state = FlashKraft::new();

        // Load drives on startup
        let initial_command = Task::perform(
            flashkraft::core::commands::load_drives(),
            Message::DrivesRefreshed,
        );

        (initial_state, initial_command)
    })
}
