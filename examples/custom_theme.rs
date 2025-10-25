//! Custom Theme Example - FlashKraft Theme System
//!
//! This example demonstrates FlashKraft's powerful theme system:
//! - 21 built-in beautiful themes
//! - Runtime theme switching
//! - Persistent theme storage
//! - Consistent theming across all components
//!
//! Run with: cargo run --example custom_theme

use flashkraft::{FlashKraft, Message};
use iced::{Settings, Task, Theme};

fn main() -> iced::Result {
    println!("===========================================");
    println!("   FlashKraft - Theme System Demo");
    println!("===========================================");
    println!();
    println!("This example showcases FlashKraft's theme system.");
    println!();
    println!("Available Themes:");
    println!("  Dark Themes:");
    println!("    • Dark (default)");
    println!("    • Dracula");
    println!("    • Nord");
    println!("    • Solarized Dark");
    println!("    • Gruvbox Dark");
    println!("    • Catppuccin Mocha");
    println!("    • Tokyo Night");
    println!("    • Kanagawa Wave");
    println!("    • Moonfly");
    println!("    • Nightfly");
    println!("    • Oxocarbon");
    println!();
    println!("  Light Themes:");
    println!("    • Light");
    println!("    • Solarized Light");
    println!("    • Gruvbox Light");
    println!("    • Catppuccin Latte");
    println!("    • Tokyo Night Light");
    println!("    • Kanagawa Lotus");
    println!();
    println!("  And more variants...");
    println!();
    println!("Usage:");
    println!("  1. Click the theme button (🎨) in the top-right");
    println!("  2. Select any theme from the picker");
    println!("  3. Watch the entire UI update instantly");
    println!("  4. Your theme preference is saved automatically");
    println!();
    println!("Features:");
    println!("  • All UI components update with theme");
    println!("  • Progress bars adapt to theme colors");
    println!("  • Buttons and panels match theme palette");
    println!("  • Status indicators themed consistently");
    println!("  • Theme persists across app restarts");
    println!();
    println!("===========================================");
    println!();

    // Start with a different theme to showcase the feature
    let mut initial_state = FlashKraft::new();
    initial_state.theme = Theme::TokyoNight;

    // Save the initial theme
    if let Some(storage) = &initial_state.storage {
        let _ = storage.save_theme(&Theme::TokyoNight);
    }

    iced::application(
        "FlashKraft - Theme System Demo",
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
        // Load drives on startup
        let initial_command = Task::perform(
            flashkraft::core::commands::load_drives(),
            Message::DrivesRefreshed,
        );

        (initial_state, initial_command)
    })
}
