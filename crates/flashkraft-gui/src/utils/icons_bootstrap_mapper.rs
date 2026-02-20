//! Bootstrap Icons Mapper Module
//!
//! This utility module provides helper functions for mapping Bootstrap Icons
//! to Iced elements in the FlashKraft application. It uses Bootstrap Icons
//! from the iced_fonts crate.

use iced::widget::text;
use iced::Element;
use iced_fonts::Bootstrap;

use crate::core::message::Message;

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
