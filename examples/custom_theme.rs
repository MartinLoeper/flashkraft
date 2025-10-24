//! Custom Theme Example - Dynamic Theme Switching
//!
//! This example demonstrates how to work with Iced's theme system:
//! - Using built-in themes
//! - Switching themes dynamically
//! - Applying themes to the entire application
//!
//! FlashKraft uses this pattern to provide 21 different themes!

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Task, Theme};

fn main() -> iced::Result {
    iced::application("Custom Theme Example", ThemeDemo::update, ThemeDemo::view)
        .theme(|state: &ThemeDemo| state.current_theme.clone())
        .window_size((600.0, 500.0))
        .run_with(|| (ThemeDemo::new(), Task::none()))
}

// ============================================================================
// MODEL - Application State
// ============================================================================

struct ThemeDemo {
    current_theme: Theme,
    counter: i32,
}

impl ThemeDemo {
    fn new() -> Self {
        Self {
            current_theme: Theme::Dark,
            counter: 0,
        }
    }

    #[allow(dead_code)]
    /// Get all available themes
    fn all_themes() -> Vec<Theme> {
        vec![
            Theme::Dark,
            Theme::Light,
            Theme::Dracula,
            Theme::Nord,
            Theme::SolarizedLight,
            Theme::SolarizedDark,
            Theme::GruvboxLight,
            Theme::GruvboxDark,
            Theme::CatppuccinLatte,
            Theme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato,
            Theme::CatppuccinMocha,
            Theme::TokyoNight,
            Theme::TokyoNightStorm,
            Theme::TokyoNightLight,
            Theme::KanagawaWave,
            Theme::KanagawaDragon,
            Theme::KanagawaLotus,
            Theme::Moonfly,
            Theme::Nightfly,
            Theme::Oxocarbon,
        ]
    }

    /// Get the name of a theme for display
    fn theme_name(theme: &Theme) -> String {
        match theme {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::SolarizedLight => "Solarized Light",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::GruvboxLight => "Gruvbox Light",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::CatppuccinLatte => "Catppuccin Latte",
            Theme::CatppuccinFrappe => "Catppuccin Frappé",
            Theme::CatppuccinMacchiato => "Catppuccin Macchiato",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
            Theme::TokyoNight => "Tokyo Night",
            Theme::TokyoNightStorm => "Tokyo Night Storm",
            Theme::TokyoNightLight => "Tokyo Night Light",
            Theme::KanagawaWave => "Kanagawa Wave",
            Theme::KanagawaDragon => "Kanagawa Dragon",
            Theme::KanagawaLotus => "Kanagawa Lotus",
            Theme::Moonfly => "Moonfly",
            Theme::Nightfly => "Nightfly",
            Theme::Oxocarbon => "Oxocarbon",
            _ => "Custom",
        }
        .to_string()
    }
}

// ============================================================================
// MESSAGE - Events
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    /// User selected a new theme
    ThemeSelected(Theme),
    /// User clicked increment
    Increment,
    /// User clicked decrement
    Decrement,
    /// User clicked reset
    Reset,
}

// ============================================================================
// UPDATE - State Transitions
// ============================================================================

impl ThemeDemo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.current_theme = theme;
            }
            Message::Increment => {
                self.counter += 1;
            }
            Message::Decrement => {
                self.counter -= 1;
            }
            Message::Reset => {
                self.counter = 0;
            }
        }
        Task::none()
    }
}

// ============================================================================
// VIEW - UI Rendering
// ============================================================================

impl ThemeDemo {
    fn view(&self) -> Element<'_, Message> {
        let title = text("Theme Switcher Demo")
            .size(32)
            .width(Length::Fill)
            .center();

        let current_theme_name = Self::theme_name(&self.current_theme);
        let theme_display = text(format!("Current theme: {}", current_theme_name))
            .size(18)
            .width(Length::Fill)
            .center();

        // Theme selector buttons (showing a few popular themes)
        let dark_btn = button(text("Dark").size(14))
            .on_press(Message::ThemeSelected(Theme::Dark))
            .padding(10);

        let light_btn = button(text("Light").size(14))
            .on_press(Message::ThemeSelected(Theme::Light))
            .padding(10);

        let dracula_btn = button(text("Dracula").size(14))
            .on_press(Message::ThemeSelected(Theme::Dracula))
            .padding(10);

        let nord_btn = button(text("Nord").size(14))
            .on_press(Message::ThemeSelected(Theme::Nord))
            .padding(10);

        let tokyo_btn = button(text("Tokyo Night").size(14))
            .on_press(Message::ThemeSelected(Theme::TokyoNight))
            .padding(10);

        let catppuccin_btn = button(text("Catppuccin Mocha").size(14))
            .on_press(Message::ThemeSelected(Theme::CatppuccinMocha))
            .padding(10);

        let theme_buttons = column![
            row![dark_btn, light_btn, dracula_btn].spacing(5),
            row![nord_btn, tokyo_btn, catppuccin_btn].spacing(5),
        ]
        .spacing(5);

        let theme_section = column![text("Select Theme:").size(16), theme_buttons, theme_display,]
            .spacing(10)
            .align_x(Alignment::Center);

        // Counter section to show theme affects all widgets
        let counter_display = text(self.counter).size(64).width(Length::Fill).center();

        let increment_button = button(text("+ Increment").size(16))
            .on_press(Message::Increment)
            .padding(12);

        let decrement_button = button(text("- Decrement").size(16))
            .on_press(Message::Decrement)
            .padding(12);

        let reset_button = button(text("Reset").size(16))
            .on_press(Message::Reset)
            .padding(12);

        let button_row = row![increment_button, decrement_button, reset_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let counter_section = column![
            text("Interactive Counter").size(20),
            text("(Watch buttons change with theme!)").size(14),
            counter_display,
            button_row,
        ]
        .spacing(15)
        .align_x(Alignment::Center);

        // Info section
        let info = column![
            text("Features:").size(18),
            text("• 21 built-in themes").size(14),
            text("• Instant theme switching").size(14),
            text("• Consistent across all widgets").size(14),
            text("• Dark and light variants").size(14),
        ]
        .spacing(8)
        .align_x(Alignment::Start);

        // Main layout
        let content = column![title, theme_section, counter_section, info]
            .spacing(30)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .padding(40);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let demo = ThemeDemo::new();
        assert!(matches!(demo.current_theme, Theme::Dark));
        assert_eq!(demo.counter, 0);
    }

    #[test]
    fn test_theme_change() {
        let mut demo = ThemeDemo::new();
        let _ = demo.update(Message::ThemeSelected(Theme::Dracula));
        assert!(matches!(demo.current_theme, Theme::Dracula));
    }

    #[test]
    fn test_counter_operations() {
        let mut demo = ThemeDemo::new();

        let _ = demo.update(Message::Increment);
        assert_eq!(demo.counter, 1);

        let _ = demo.update(Message::Increment);
        assert_eq!(demo.counter, 2);

        let _ = demo.update(Message::Decrement);
        assert_eq!(demo.counter, 1);

        let _ = demo.update(Message::Reset);
        assert_eq!(demo.counter, 0);
    }

    #[test]
    fn test_all_themes_available() {
        let themes = ThemeDemo::all_themes();
        assert_eq!(themes.len(), 21);
    }

    #[test]
    fn test_theme_names() {
        assert_eq!(ThemeDemo::theme_name(&Theme::Dark), "Dark");
        assert_eq!(ThemeDemo::theme_name(&Theme::Light), "Light");
        assert_eq!(ThemeDemo::theme_name(&Theme::Dracula), "Dracula");
        assert_eq!(ThemeDemo::theme_name(&Theme::Nord), "Nord");
    }
}

// ============================================================================
// KEY TAKEAWAYS
// ============================================================================
//
// 1. THEME SYSTEM:
//    - Iced provides 21 built-in themes
//    - Themes affect all widgets consistently
//    - Easy to switch at runtime
//
// 2. IMPLEMENTING THEMES:
//    - Store current theme in model
//    - Use .theme() method on application
//    - Return theme from a function taking &State
//
// 3. DYNAMIC SWITCHING:
//    - Theme changes trigger full re-render
//    - No need to manually update colors
//    - Iced handles the details
//
// 4. CUSTOM THEMES:
//    - Can create custom themes with Theme::custom()
//    - Define your own color palette
//    - Maintain consistency across app
//
// 5. BEST PRACTICES:
//    - Provide theme selector in settings
//    - Save user preference (not shown here)
//    - Test with light and dark themes
//    - Ensure readability in all themes
//
// 6. USAGE IN FLASHKRAFT:
//    - FlashKraft uses this exact pattern
//    - Uses button-based theme selector
//    - Could use pick_list with wrapper type to avoid orphan rules
//    - Persists across sessions
//    - Enhances user experience
//
// Try clicking the theme buttons to see how the entire UI adapts instantly!
