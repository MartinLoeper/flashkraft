//! View Logic - The Elm Architecture
//!
//! This module contains all the view functions that render the UI
//! based on the application state. Views are pure functions that
//! take state and return a description of what to display.

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::widget::{
    button, canvas, column, container, pick_list, progress_bar, row, scrollable, text, Space,
};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::icons;
use crate::message::Message;
use crate::model::{DriveInfo, FlashKraft, ImageInfo};
use iced_fonts::Bootstrap;

// ============================================================================
// Main View Entry Point
// ============================================================================

/// Main view function - decides what to show based on state
///
/// This is the entry point for rendering the UI. It examines the
/// current state and delegates to the appropriate view function.
///
/// # Arguments
///
/// * `state` - The current application state
///
/// # Returns
///
/// An Element describing the UI to render
pub fn view(state: &FlashKraft) -> Element<'_, Message> {
    let content = if state.is_flash_complete() {
        // Flash completed successfully
        view_complete()
    } else if state.is_flashing() {
        // Currently flashing
        view_flashing(state)
    } else if state.has_error() {
        // Error occurred
        let error = state.error_message.as_deref().unwrap_or("Unknown error");
        view_error(error)
    } else {
        // Normal main view
        view_main(state)
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

// ============================================================================
// Main Application View
// ============================================================================

/// Main application view (normal state)
fn view_main(state: &FlashKraft) -> Element<'_, Message> {
    // If device selection is open, show it as an overlay
    if state.device_selection_open {
        return view_device_selector(state);
    }

    let header = view_header(state);
    let step_indicators = view_step_indicators(state);
    let buttons = view_buttons(state);

    column![header, step_indicators, buttons]
        .spacing(30)
        .padding(20)
        .width(Length::Fill)
        .into()
}

/// Application header
fn view_header(state: &FlashKraft) -> Element<'_, Message> {
    // All available themes in iced
    let all_themes = vec![
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
    ];

    let theme_picker = pick_list(all_themes, Some(state.theme.clone()), Message::ThemeChanged)
        .placeholder("Select theme")
        .text_size(14.0)
        .width(Length::Fixed(200.0));

    container(
        row![
            row![
                icons::icon(Bootstrap::LightningFill, 32.0),
                text(" FlashKraft").size(32)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            Space::with_width(Length::Fill),
            theme_picker,
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(20)
    .into()
}

/// Step indicators with connecting lines (Balena Etcher style)
fn view_step_indicators(state: &FlashKraft) -> Element<'_, Message> {
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

    // Create connecting lines - longer lines to properly connect icons
    let line1 = view_progress_line(has_image, 100.0);
    let line2 = view_progress_line(has_target, 100.0);

    container(row![step1, line1, step2, line2, step3].align_y(Alignment::Center))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Draw a progress line between steps
fn view_progress_line(completed: bool, width: f32) -> Element<'static, Message> {
    let color = if completed {
        Color::from_rgb(0.3, 0.8, 0.3)
    } else {
        Color::from_rgb(0.5, 0.5, 0.5)
    };

    container(
        canvas(ProgressLine { color, width })
            .width(width)
            .height(4.0),
    )
    .center_y(Length::Fill)
    .padding(iced::Padding {
        top: 0.0,
        right: 0.0,
        bottom: 30.0,
        left: 0.0,
    })
    .into()
}

struct ProgressLine {
    color: Color,
    width: f32,
}

impl canvas::Program<Message> for ProgressLine {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let line = Path::line(
            iced::Point::new(0.0, bounds.height / 2.0),
            iced::Point::new(self.width, bounds.height / 2.0),
        );

        frame.stroke(
            &line,
            Stroke::default().with_color(self.color).with_width(4.0),
        );

        vec![frame.into_geometry()]
    }
}

/// Three buttons section
fn view_buttons(state: &FlashKraft) -> Element<'_, Message> {
    let image_button = view_image_section(&state.selected_image);
    let target_button = view_target_section(&state.selected_target);
    let flash_button = view_flash_section(state.is_ready_to_flash());

    let buttons_row = row![image_button, target_button, flash_button]
        .spacing(40)
        .align_y(Alignment::Start);

    let mut content = column![buttons_row].spacing(20).align_x(Alignment::Center);

    // Show cancel button if something is selected
    if state.selected_image.is_some() || state.selected_target.is_some() {
        content = content.push(
            button(text("Cancel").size(14))
                .on_press(Message::CancelClicked)
                .padding(10),
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

// ============================================================================
// Selection Panels
// ============================================================================

/// Image selection panel
fn view_image_section(image: &Option<ImageInfo>) -> Element<'_, Message> {
    let content = if let Some(img) = image {
        column![
            icons::icon(Bootstrap::CheckCircle, 50.0),
            text(&img.name).size(16),
            text({
                if img.size_mb >= 1024.0 {
                    format!("{:.2} GB", img.size_mb / 1024.0)
                } else {
                    format!("{:.2} MB", img.size_mb)
                }
            })
            .size(12),
        ]
    } else {
        column![
            icons::icon(Bootstrap::PlusCircle, 50.0),
            text("Flash from file").size(16),
            text("Click to select").size(12),
        ]
    };

    button(
        container(content.spacing(10).align_x(Alignment::Center))
            .width(220)
            .height(90)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(Message::SelectImageClicked)
    .padding(10)
    .into()
}

/// Target drive selection panel
fn view_target_section(selected: &Option<DriveInfo>) -> Element<'_, Message> {
    let content = if let Some(target) = selected {
        column![
            icons::icon(Bootstrap::CheckCircle, 50.0),
            text(&target.name).size(16),
            text({
                if target.size_gb >= 1.0 {
                    format!("{:.2} GB", target.size_gb)
                } else if target.size_gb >= 0.001 {
                    format!("{:.2} MB", target.size_gb * 1024.0)
                } else {
                    format!("{:.2} KB", target.size_gb * 1024.0 * 1024.0)
                }
            })
            .size(12),
            text(&target.mount_point).size(10),
        ]
    } else {
        column![
            icons::icon(Bootstrap::DeviceHdd, 50.0),
            text("Select target").size(16),
            text("Click to choose").size(12),
        ]
    };

    button(
        container(content.spacing(10).align_x(Alignment::Center))
            .width(220)
            .height(90)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(Message::OpenDeviceSelection)
    .padding(10)
    .into()
}

/// Flash button panel
fn view_flash_section(is_ready: bool) -> Element<'static, Message> {
    let content = column![
        icons::icon(Bootstrap::LightningFill, 50.0),
        text("Flash!").size(16),
        text(if is_ready {
            "Ready to flash"
        } else {
            "Select image & target"
        })
        .size(12),
    ];

    let btn = button(
        container(content.spacing(10).align_x(Alignment::Center))
            .width(220)
            .height(90)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(10);

    if is_ready {
        btn.on_press(Message::FlashClicked).into()
    } else {
        btn.into()
    }
}

// ============================================================================
// Device Selection View
// ============================================================================

/// Device selector overlay
fn view_device_selector(state: &FlashKraft) -> Element<'_, Message> {
    let title = text("Select Target Drive").size(24);

    let drives_list: Element<'_, Message> = if state.available_drives.is_empty() {
        column![text("No drives detected").size(16)]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
    } else {
        let drive_rows: Vec<Element<'_, Message>> = state
            .available_drives
            .iter()
            .map(|drive| view_device_row(drive, state.selected_target.as_ref()))
            .collect();

        scrollable(column(drive_rows).spacing(10))
            .height(Length::Fill)
            .into()
    };

    let refresh_button = button(
        row![
            icons::icon(Bootstrap::ArrowClockwise, 16.0),
            text("Refresh").size(14)
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(Message::RefreshDrivesClicked)
    .padding(10);

    let cancel_button = button(text("Cancel").size(14))
        .on_press(Message::CloseDeviceSelection)
        .padding(10);

    let content = column![
        title,
        Space::with_height(20),
        drives_list,
        Space::with_height(20),
        row![refresh_button, Space::with_width(10), cancel_button].align_y(Alignment::Center)
    ]
    .spacing(10)
    .padding(30)
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Single drive row in device selector
fn view_device_row<'a>(
    drive: &'a DriveInfo,
    selected: Option<&'a DriveInfo>,
) -> Element<'a, Message> {
    let is_selected = selected.map_or(false, |s| s.device_path == drive.device_path);

    let icon = if is_selected {
        icons::icon(Bootstrap::CheckCircleFill, 40.0)
    } else {
        icons::icon(Bootstrap::DeviceHdd, 40.0)
    };

    let name_text = text(&drive.name).size(18);
    let size_text = text(format!("{:.2} GB", drive.size_gb)).size(14);
    let mount_text = text(&drive.mount_point).size(12);
    let device_text = text(&drive.device_path).size(12);

    let info = column![
        name_text,
        row![size_text, text(" • ").size(14), mount_text].spacing(5),
        device_text
    ]
    .spacing(5);

    button(container(row![icon, info].spacing(20).align_y(Alignment::Center)).width(Length::Fill))
        .on_press(Message::TargetDriveClicked(drive.clone()))
        .padding(5)
        .into()
}

// ============================================================================
// Status Views
// ============================================================================

/// Flashing progress view
fn view_flashing(state: &FlashKraft) -> Element<'_, Message> {
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

    let content = column![
        icons::icon(Bootstrap::LightningFill, 80.0),
        text(format!("Flashing... {}%", progress_percent)).size(32),
        Space::with_height(20),
        progress_bar(0.0..=1.0, progress),
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

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Error view
fn view_error(error: &str) -> Element<'_, Message> {
    let content = column![
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

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Flash complete view
fn view_complete() -> Element<'static, Message> {
    let content = column![
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

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_renders() {
        let state = FlashKraft::new();
        let _view = view(&state);
        // If this compiles and runs, the view renders successfully
    }
}
