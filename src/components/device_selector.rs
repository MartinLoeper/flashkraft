//! Device Selector Component
//!
//! This module contains the device selector overlay that allows users
//! to choose a target drive for flashing.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::core::message::Message;
use crate::domain::DriveInfo;
use crate::utils::icons_bootstrap_mapper as icons;
use iced_fonts::Bootstrap;

/// Device selector overlay
pub fn view_device_selector<'a>(
    available_drives: &'a [DriveInfo],
    selected_target: &'a Option<DriveInfo>,
) -> Element<'a, Message> {
    let title = text("Select Target Drive").size(24);

    let drives_list: Element<'_, Message> = if available_drives.is_empty() {
        column![text("No drives detected").size(16)]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
    } else {
        let drive_rows: Vec<Element<'_, Message>> = available_drives
            .iter()
            .map(|drive| view_device_row(drive, selected_target.as_ref()))
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
    let is_selected = selected.is_some_and(|s| s.device_path == drive.device_path);

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
