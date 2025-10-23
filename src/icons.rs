//! Icons Module
//!
//! This module provides helper functions and constants for using icons
//! in the FlashKraft application. It uses Bootstrap Icons from iced_fonts.

use iced::widget::text;
use iced::Element;
use iced_fonts::Bootstrap;

use crate::message::Message;

/// Create an icon element from a Bootstrap icon
///
/// # Arguments
///
/// * `icon` - The Bootstrap icon to display
/// * `size` - The size of the icon in pixels
///
/// # Returns
///
/// An Element that displays the icon
pub fn icon<'a>(icon: Bootstrap, size: f32) -> Element<'a, Message> {
    text(char::from(icon))
        .font(iced_fonts::BOOTSTRAP_FONT)
        .size(size)
        .into()
}
