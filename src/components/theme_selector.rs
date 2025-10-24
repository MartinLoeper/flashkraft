//! Theme Selector Component
//!
//! This module provides a reusable theme selector component that displays
//! all available themes and handles theme changes with persistence.

use crate::core::message::Message;
use iced::widget::{container, pick_list};
use iced::{Alignment, Element, Length, Theme};

/// Get all available themes in Iced
pub fn all_themes() -> Vec<Theme> {
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

/// Create a theme selector widget
///
/// # Arguments
///
/// * `current_theme` - The currently selected theme
///
/// # Returns
///
/// An Element containing the theme picker
pub fn theme_selector(current_theme: &Theme) -> Element<'static, Message> {
    let themes = all_themes();

    pick_list(themes, Some(current_theme.clone()), Message::ThemeChanged)
        .placeholder("Select theme")
        .text_size(14.0)
        .width(Length::Fixed(200.0))
        .into()
}

/// Create a theme selector widget aligned to the right
///
/// # Arguments
///
/// * `current_theme` - The currently selected theme
///
/// # Returns
///
/// An Element containing the right-aligned theme picker
pub fn theme_selector_right(current_theme: &Theme) -> Element<'static, Message> {
    use iced::widget::{row, Space};

    container(
        row![
            Space::with_width(Length::Fill),
            theme_selector(current_theme),
        ]
        .align_y(Alignment::Start),
    )
    .padding(20)
    .into()
}
