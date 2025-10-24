//! Header Component
//!
//! This module contains the application header with title and theme selector.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::components::theme_selector;
use crate::core::message::Message;
use crate::core::FlashKraft;
use crate::utils::icons_bootstrap_mapper as icons;
use iced_fonts::Bootstrap;

/// Application header with title and theme selector
pub fn view_header(state: &FlashKraft) -> Element<'_, Message> {
    // Centered title with larger text
    let title_content = container(
        column![container(
            row![
                icons::icon(Bootstrap::LightningFill, 48.0),
                Space::with_width(20),
                text("FlashKraft").size(56).style(move |theme: &Theme| {
                    let palette = theme.palette();
                    iced::widget::text::Style {
                        color: Some(Color::from_rgb(
                            (palette.primary.r * 1.2).min(1.0),
                            (palette.primary.g * 1.2).min(1.0),
                            (palette.primary.b * 1.2).min(1.0),
                        )),
                    }
                }),
            ]
            .align_y(Alignment::Center),
        )
        .center_x(Length::Fill),]
        .spacing(0),
    )
    .width(Length::Fill);

    column![
        theme_selector::theme_selector_right(&state.theme),
        title_content,
    ]
    .into()
}
