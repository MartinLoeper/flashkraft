//! Status Views Component
//!
//! This module contains the various status views for the application:
//! - Flashing progress view
//! - Error view
//! - Flash complete view

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::components::theme_selector;
use crate::core::message::Message;
use crate::core::FlashKraft;
use crate::utils::icons_bootstrap_mapper as icons;
use iced_fonts::Bootstrap;

/// Flashing progress view
pub fn view_flashing(state: &FlashKraft) -> Element<'_, Message> {
    let progress = state.flash_progress.unwrap_or(0.0);
    let progress_percent = (progress * 100.0) as u32;
    let speed_mb_s = state.flash_speed_mb_s;

    // Calculate ETA
    let eta_text = if speed_mb_s > 0.0 && progress > 0.0 {
        let bytes_written = state.flash_bytes_written;
        let total_bytes = state
            .selected_image
            .as_ref()
            .map(|img| (img.size_mb * 1024.0 * 1024.0) as u64)
            .unwrap_or(0);
        let bytes_remaining = total_bytes.saturating_sub(bytes_written);
        let speed_bytes_s = speed_mb_s * 1024.0 * 1024.0;
        let eta_seconds = (bytes_remaining as f32 / speed_bytes_s) as u64;

        let minutes = eta_seconds / 60;
        let seconds = eta_seconds % 60;
        format!("ETA: {}m{}s", minutes, seconds)
    } else {
        "ETA: calculating...".to_string()
    };

    let speed_text = if speed_mb_s > 0.0 {
        format!("{:.1} MB/s", speed_mb_s)
    } else {
        "-- MB/s".to_string()
    };

    let progress_content = column![
        icons::icon(Bootstrap::LightningFill, 80.0),
        text(format!("Flashing... {}%", progress_percent)).size(32),
        Space::with_height(20),
        state
            .animated_progress
            .view::<Message>()
            .map(|_| Message::AnimationTick),
        Space::with_height(15),
        row![
            text(speed_text).size(16),
            Space::with_width(40),
            text(eta_text).size(16),
        ]
        .align_y(Alignment::Center),
        Space::with_height(20),
        button(text("Cancel").size(14))
            .on_press(Message::CancelFlash)
            .padding(10),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(40);

    let content = column![
        theme_selector::theme_selector_right(&state.theme),
        container(progress_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Error view
pub fn view_error<'a>(state: &'a crate::core::FlashKraft, error: &'a str) -> Element<'a, Message> {
    let error_content = column![
        icons::icon(Bootstrap::ExclamationTriangleFill, 80.0),
        text("Error").size(32),
        Space::with_height(20),
        text(error).size(16),
        Space::with_height(20),
        button(text("Try Again").size(14))
            .on_press(Message::ResetClicked)
            .padding(10),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(40);

    let content = column![
        theme_selector::theme_selector_right(&state.theme),
        container(error_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Flash complete view
pub fn view_complete(state: &crate::core::FlashKraft) -> Element<'_, Message> {
    let complete_content = column![
        icons::icon(Bootstrap::CheckCircleFill, 80.0),
        text("Flash Complete!").size(32),
        Space::with_height(20),
        text("Your device is ready to use").size(16),
        Space::with_height(20),
        button(text("Flash Another").size(14))
            .on_press(Message::ResetClicked)
            .padding(10),
    ]
    .spacing(10)
    .align_x(Alignment::Center)
    .padding(40);

    let content = column![
        theme_selector::theme_selector_right(&state.theme),
        container(complete_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
