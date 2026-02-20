//! Step Indicators Component
//!
//! This module contains the step indicator UI that shows the three main steps
//! (Select Image, Select Target, Flash) with connecting animated progress lines.

use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::components::progress_line;
use crate::core::message::Message;
use crate::core::FlashKraft;
use crate::utils::icons_bootstrap_mapper as icons;
use iced_fonts::Bootstrap;

/// Step indicators with connecting lines (Balena Etcher style)
pub fn view_step_indicators(state: &FlashKraft) -> Element<'_, Message> {
    let has_image = state.selected_image.is_some();
    let has_target = state.selected_target.is_some();

    // Create step indicators - width matches button content width (220px)
    let step1 = container(
        column![
            icons::icon(Bootstrap::Image, 32.0),
            text("Select Image").size(13),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(220)
    .center_x(220);

    let step2 = container(
        column![
            icons::icon(Bootstrap::DeviceHdd, 32.0),
            text("Select Target").size(13),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(220)
    .center_x(220);

    let step3 = container(
        column![
            icons::icon(Bootstrap::LightningFill, 32.0),
            text("Flash!").size(13),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(220)
    .center_x(220);

    // Create connecting lines - animated with theme colors
    let line1 =
        progress_line::view_progress_line(has_image, 200.0, state.animation_time, &state.theme);
    let line2 =
        progress_line::view_progress_line(has_target, 200.0, state.animation_time, &state.theme);

    container(row![step1, line1, step2, line2, step3].align_y(Alignment::Center))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
