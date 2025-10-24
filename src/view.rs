//! View Logic - The Elm Architecture
//!
//! This module contains the main view function that renders the UI
//! based on the application state. Views are pure functions that
//! take state and return a description of what to display.
//!
//! Most view logic has been extracted into component modules for better organization.

use iced::widget::{column, container};
use iced::{Element, Length};

use crate::components::{device_selector, header, selection_panels, status_views, step_indicators};
use crate::core::{FlashKraft, Message};

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
        status_views::view_complete()
    } else if state.is_flashing() {
        // Currently flashing
        status_views::view_flashing(state)
    } else if state.has_error() {
        // Error occurred
        let error = state.error_message.as_deref().unwrap_or("Unknown error");
        status_views::view_error(error)
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
        return device_selector::view_device_selector(
            &state.available_drives,
            &state.selected_target,
        );
    }

    let header = header::view_header(state);
    let step_indicators = step_indicators::view_step_indicators(state);
    let buttons = selection_panels::view_buttons(
        &state.selected_image,
        &state.selected_target,
        state.is_ready_to_flash(),
    );

    column![header, step_indicators, buttons]
        .spacing(30)
        .padding(20)
        .width(Length::Fill)
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
