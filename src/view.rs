//! View Logic - The Elm Architecture
//!
//! This module contains all the view functions that render the UI
//! based on the application state. Views are pure functions that
//! take state and return a description of what to display.

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::widget::{button, canvas, column, container, progress_bar, row, scrollable, text, Space};
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
        let progress = state.flash_progress.unwrap_or(0.0);
        view_flashing(progress)
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

    let header = view_header();
    let step_indicator = view_step_indicator(state);
    let main_section = view_main_section(state);

    column![header, step_indicator, main_section]
        .spacing(30)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Application header
fn view_header() -> Element<'static, Message> {
    container(
        row![
            icons::icon(Bootstrap::LightningFill, 32.0),
            text(" FlashKraft").size(32)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(20)
    .into()
}

/// Step indicator showing progress through the workflow with animated lines
fn view_step_indicator(state: &FlashKraft) -> Element<'_, Message> {
    let has_image = state.selected_image.is_some();
    let has_target = state.selected_target.is_some();
    let is_ready = state.is_ready_to_flash();

    let step1 = column![
        icons::icon(
            if has_image {
                Bootstrap::CheckCircle
            } else {
                Bootstrap::Image
            },
            40.0
        ),
        text("Select Image").size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center);

    let line1 = view_progress_line(
        if has_image {
            Color::from_rgb(0.3, 0.8, 0.3)
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        },
        has_image,
    );

    let step2 = column![
        icons::icon(
            if has_target {
                Bootstrap::CheckCircle
            } else {
                Bootstrap::DeviceHdd
            },
            40.0
        ),
        text("Select Target").size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center);

    let line2 = view_progress_line(
        if has_target {
            Color::from_rgb(0.3, 0.8, 0.3)
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        },
        has_target,
    );

    let step3 = column![
        icons::icon(
            if is_ready {
                Bootstrap::CheckCircle
            } else {
                Bootstrap::LightningFill
            },
            40.0
        ),
        text("Flash!").size(14),
    ]
    .spacing(10)
    .align_x(Alignment::Center);

    container(
        row![step1, line1, step2, line2, step3,]
            .spacing(20)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}

/// Draw an animated progress line
fn view_progress_line(color: Color, _completed: bool) -> Element<'static, Message> {
    container(
        canvas(ProgressLine {
            color,
            width: 100.0,
        })
        .width(100.0)
        .height(4.0),
    )
    .center_y(Length::Fill)
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

/// Main section with three panels
fn view_main_section(state: &FlashKraft) -> Element<'_, Message> {
    let image_section = view_image_section(&state.selected_image);
    let target_section = view_target_section(&state.selected_target);
    let flash_section = view_flash_section(state.is_ready_to_flash());

    let main_row = row![image_section, target_section, flash_section]
        .spacing(40)
        .align_y(Alignment::Start);

    let mut content = column![main_row].spacing(20).align_x(Alignment::Center);

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
            .width(250)
            .height(150)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(Message::SelectImageClicked)
    .padding(20)
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
            .width(250)
            .height(150)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(Message::OpenDeviceSelection)
    .padding(20)
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
            .width(250)
            .height(150)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(20);

    if is_ready {
        btn.on_press(Message::FlashClicked).into()
    } else {
        btn.into()
    }
}

// ============================================================================
// Device Selection Modal
// ============================================================================

/// Device selector modal view
fn view_device_selector(state: &FlashKraft) -> Element<'_, Message> {
    let header = row![
        text("FlashKraft").size(46),
        Space::with_width(Length::Fill),
        button(icons::icon(Bootstrap::X, 20.0))
            .on_press(Message::CloseDeviceSelection)
            .padding(10),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Table header
    let table_header = row![
        container(text("Name").size(14))
            .width(Length::FillPortion(3))
            .padding(10),
        container(text("Size").size(14))
            .width(Length::FillPortion(2))
            .padding(10),
        container(text("Location").size(14))
            .width(Length::FillPortion(3))
            .padding(10),
        container(text("").size(14))
            .width(Length::FillPortion(2))
            .padding(10),
    ]
    .spacing(5)
    .width(Length::Fill);

    // Device rows
    let device_rows: Vec<Element<'_, Message>> = state
        .available_drives
        .iter()
        .map(|drive| view_device_row(drive, state))
        .collect();

    let devices_list: Element<'_, Message> = if device_rows.is_empty() {
        container(
            column![
                text("No drives detected").size(16),
                text("Please insert a USB drive or SD card").size(12),
                button(text("Refresh").size(14))
                    .on_press(Message::RefreshDrivesClicked)
                    .padding(10)
            ]
            .spacing(15)
            .align_x(Alignment::Center)
            .padding(40),
        )
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
    } else {
        column(device_rows).spacing(2).width(Length::Fill).into()
    };

    // Footer with action buttons
    let selected_count = if state.selected_target.is_some() {
        1
    } else {
        0
    };
    let footer = row![
        button(text("Cancel").size(14))
            .on_press(Message::CloseDeviceSelection)
            .padding(12),
        Space::with_width(Length::Fill),
        button(text(format!("Select {}", selected_count)).size(14)).padding(12),
    ]
    .spacing(20)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // Main modal container
    container(
        column![
            header,
            Space::with_height(20),
            container(table_header)
                .style(container::bordered_box)
                .padding(5),
            Space::with_height(2),
            container(scrollable(devices_list).height(Length::Fixed(400.0)))
                .style(container::bordered_box)
                .padding(10),
            Space::with_height(20),
            footer,
        ]
        .spacing(0)
        .width(Length::Fixed(800.0))
        .padding(20),
    )
    .style(container::bordered_box)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Individual device row in the selector table
fn view_device_row<'a>(drive: &'a DriveInfo, state: &'a FlashKraft) -> Element<'a, Message> {
    // Determine status badges
    let mut status_badges: Vec<Element<'_, Message>> = Vec::new();

    // Check if drive is too small (less than 1 GB)
    if drive.size_gb < 1.0 {
        status_badges.push(
            container(text("Too small").size(10))
                .style(container::bordered_box)
                .padding(4)
                .into(),
        );
    }

    // Check if it's a system drive (contains certain paths)
    let is_system = drive.mount_point.contains("/home")
        || drive.mount_point == "/"
        || drive.mount_point.contains("SWAP");
    if is_system {
        status_badges.push(
            container(text("System drive").size(10))
                .style(container::bordered_box)
                .padding(4)
                .into(),
        );
    }

    // Check if it's the source drive (if image is on this drive)
    if let Some(ref img) = state.selected_image {
        if img.path.starts_with(&drive.mount_point) {
            status_badges.push(
                container(text("Source drive").size(10))
                    .style(container::bordered_box)
                    .padding(4)
                    .into(),
            );
        }
    }

    let status_column: Element<'_, Message> = if status_badges.is_empty() {
        container(text("").size(10))
            .width(Length::FillPortion(2))
            .padding(10)
            .into()
    } else {
        container(row(status_badges).spacing(5))
            .width(Length::FillPortion(2))
            .padding(10)
            .into()
    };

    let warning_element: Element<'_, Message> = if is_system {
        icons::icon(Bootstrap::ExclamationTriangle, 14.0)
    } else {
        Space::with_width(0).into()
    };

    let row_element: Element<'_, Message> = row![
        container(
            row![warning_element, text(&drive.name).size(14),]
                .spacing(5)
                .align_y(Alignment::Center)
        )
        .width(Length::FillPortion(3))
        .padding(10),
        container(
            text({
                if drive.size_gb >= 1.0 {
                    format!("{:.2} GB", drive.size_gb)
                } else if drive.size_gb >= 0.001 {
                    format!("{:.2} MB", drive.size_gb * 1024.0)
                } else {
                    format!("{:.2} KB", drive.size_gb * 1024.0 * 1024.0)
                }
            })
            .size(14)
        )
        .width(Length::FillPortion(2))
        .padding(10),
        container(text(&drive.mount_point).size(12))
            .width(Length::FillPortion(3))
            .padding(10),
        status_column,
    ]
    .spacing(5)
    .width(Length::Fill)
    .into();

    button(row_element)
        .width(Length::Fill)
        .on_press(Message::TargetDriveClicked(drive.clone()))
        .into()
}

// ============================================================================
// State-Specific Views
// ============================================================================

/// Flashing in progress view
fn view_flashing(progress: f32) -> Element<'static, Message> {
    let progress_percent = (progress * 100.0) as u32;

    // If progress is stuck at 0, show a more helpful message
    let (progress_text, status_text) = if progress_percent == 0 {
        (
            "Flashing in progress...".to_string(),
            "Check the terminal window for real-time progress updates",
        )
    } else {
        (
            format!("Flashing... {}%", progress_percent),
            "Do not disconnect the target device",
        )
    };

    container(
        column![
            icons::icon(Bootstrap::LightningFill, 80.0),
            text(progress_text).size(24),
            progress_bar(0.0..=1.0, progress).width(400).height(20),
            text(status_text).size(14),
            text("This may take several minutes").size(12),
            text("Progress is shown in the terminal window")
                .size(11)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
        ]
        .spacing(20)
        .align_x(Alignment::Center)
        .padding(40),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Error view
fn view_error(error: &str) -> Element<'_, Message> {
    container(
        column![
            icons::icon(Bootstrap::ExclamationTriangleFill, 80.0),
            text("Error").size(32),
            text(error).size(16),
            button(text("Try Again").size(16))
                .on_press(Message::ResetClicked)
                .padding(15)
        ]
        .spacing(20)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// View for flash complete state
fn view_complete() -> Element<'static, Message> {
    container(
        column![
            icons::icon(Bootstrap::CheckCircleFill, 80.0),
            text("Flash Complete!").size(32),
            text("You can safely remove the target device").size(16),
            button(text("Flash Another").size(16))
                .on_press(Message::ResetClicked)
                .padding(15)
        ]
        .spacing(20)
        .align_x(Alignment::Center),
    )
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
        let element = view(&state);
        // Just verify it doesn't crash
        let _ = element;
    }
}
