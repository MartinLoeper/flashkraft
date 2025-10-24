//! Basic Usage Example - FlashKraft UI Patterns
//!
//! This example demonstrates the core UI patterns used in FlashKraft:
//! - File selection workflow
//! - Device selection
//! - Multi-step process
//! - State management
//!
//! This is a simplified version of FlashKraft that shows the essential
//! patterns without the complexity of actual disk operations.

use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Task};
use std::path::PathBuf;

fn main() -> iced::Result {
    iced::application(
        "Basic Usage - FlashKraft Patterns",
        SimplifiedFlasher::update,
        SimplifiedFlasher::view,
    )
    .window_size((700.0, 500.0))
    .run_with(|| (SimplifiedFlasher::new(), Task::none()))
}

// ============================================================================
// MODEL - Application State
// ============================================================================

/// Simplified FlashKraft state
struct SimplifiedFlasher {
    /// Selected image file
    selected_image: Option<ImageFile>,
    /// Selected target device
    selected_device: Option<Device>,
    /// Available devices
    available_devices: Vec<Device>,
    /// Status message
    status: String,
}

/// Information about an image file
#[derive(Debug, Clone)]
struct ImageFile {
    name: String,
    #[allow(dead_code)]
    path: PathBuf,
    size_mb: f64,
}

/// Information about a storage device
#[derive(Debug, Clone)]
struct Device {
    name: String,
    size_gb: f64,
    path: String,
}

impl SimplifiedFlasher {
    fn new() -> Self {
        // Pre-populate with some mock devices
        let devices = vec![
            Device {
                name: "USB Drive".to_string(),
                size_gb: 32.0,
                path: "/dev/sdb".to_string(),
            },
            Device {
                name: "SD Card".to_string(),
                size_gb: 16.0,
                path: "/dev/sdc".to_string(),
            },
            Device {
                name: "External SSD".to_string(),
                size_gb: 256.0,
                path: "/dev/sdd".to_string(),
            },
        ];

        Self {
            selected_image: None,
            selected_device: None,
            available_devices: devices,
            status: "Ready to flash".to_string(),
        }
    }

    /// Check if ready to proceed
    fn is_ready(&self) -> bool {
        self.selected_image.is_some() && self.selected_device.is_some()
    }
}

// ============================================================================
// MESSAGE - Events
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    /// User clicked "Select Image"
    SelectImageClicked,
    /// Image was selected (simulated)
    ImageSelected,
    /// User selected a specific device
    DeviceSelected(Device),
    /// User clicked "Flash"
    FlashClicked,
    /// User clicked "Reset"
    ResetClicked,
}

// ============================================================================
// UPDATE - State Transitions
// ============================================================================

impl SimplifiedFlasher {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectImageClicked => {
                // In real app: use rfd::AsyncFileDialog
                // For demo: immediately select a mock file
                Task::perform(async {}, |_| Message::ImageSelected)
            }

            Message::ImageSelected => {
                // Simulate selecting an image file
                self.selected_image = Some(ImageFile {
                    name: "ubuntu-22.04-desktop-amd64.iso".to_string(),
                    path: PathBuf::from("/home/user/Downloads/ubuntu-22.04-desktop-amd64.iso"),
                    size_mb: 3800.0,
                });
                self.status = "Image selected. Now select a target device.".to_string();
                Task::none()
            }

            Message::DeviceSelected(device) => {
                self.selected_device = Some(device.clone());
                self.status = format!("Selected: {}. Ready to flash!", device.name);
                Task::none()
            }

            Message::FlashClicked => {
                if self.is_ready() {
                    self.status = "✓ Flash operation simulated successfully!".to_string();
                }
                Task::none()
            }

            Message::ResetClicked => {
                self.selected_image = None;
                self.selected_device = None;
                self.status = "Ready to flash".to_string();
                Task::none()
            }
        }
    }
}

// ============================================================================
// VIEW - UI Rendering
// ============================================================================

impl SimplifiedFlasher {
    fn view(&self) -> Element<'_, Message> {
        let title = text("FlashKraft - Basic Usage Example")
            .size(28)
            .width(Length::Fill)
            .center();

        let subtitle = text("A simplified demonstration of FlashKraft's UI patterns")
            .size(14)
            .width(Length::Fill)
            .center();

        // Status bar
        let status_text = text(&self.status).size(16);

        // Step 1: Image Selection
        let image_section = self.view_image_section();

        // Step 2: Device Selection
        let device_section = self.view_device_section();

        // Step 3: Action buttons
        let action_buttons = self.view_action_buttons();

        let content = column![
            title,
            subtitle,
            status_text,
            image_section,
            device_section,
            action_buttons,
        ]
        .spacing(20)
        .padding(30)
        .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// View for image selection section
    fn view_image_section(&self) -> Element<'_, Message> {
        let header = text("1. Select Image").size(20);

        let content = if let Some(image) = &self.selected_image {
            // Image is selected - show details
            column![
                text("✓ Image selected").size(16),
                text(&image.name).size(14),
                text(format!("Size: {:.1} MB", image.size_mb)).size(12),
            ]
            .spacing(5)
        } else {
            // No image selected - show button
            column![
                text("No image selected").size(14),
                button(text("📁 Select Image File").size(16))
                    .on_press(Message::SelectImageClicked)
                    .padding(12),
            ]
            .spacing(10)
        };

        column![header, content]
            .spacing(10)
            .padding(15)
            .width(Length::Fill)
            .into()
    }

    /// View for device selection section
    fn view_device_section(&self) -> Element<'_, Message> {
        let header = text("2. Select Target Device").size(20);

        let selected_info = if let Some(device) = &self.selected_device {
            text(format!(
                "✓ Selected: {} ({:.1} GB)",
                device.name, device.size_gb
            ))
            .size(14)
        } else {
            text("No device selected").size(14)
        };

        // List of available devices
        let device_list: Element<_> = self
            .available_devices
            .iter()
            .fold(column![].spacing(5), |col, device| {
                let is_selected = self
                    .selected_device
                    .as_ref()
                    .map(|d| d.path == device.path)
                    .unwrap_or(false);

                let label = if is_selected {
                    format!("✓ {} - {:.1} GB", device.name, device.size_gb)
                } else {
                    format!("  {} - {:.1} GB", device.name, device.size_gb)
                };

                let btn = button(text(label).size(14))
                    .on_press(Message::DeviceSelected(device.clone()))
                    .padding(10)
                    .width(Length::Fill);

                col.push(btn)
            })
            .into();

        column![header, selected_info, device_list]
            .spacing(10)
            .padding(15)
            .width(Length::Fill)
            .into()
    }

    /// View for action buttons
    fn view_action_buttons(&self) -> Element<'_, Message> {
        let flash_button = if self.is_ready() {
            button(text("⚡ Flash!").size(20))
                .on_press(Message::FlashClicked)
                .padding(15)
        } else {
            button(text("⚡ Flash!").size(20)).padding(15)
        };

        let reset_button = button(text("🔄 Reset").size(16))
            .on_press(Message::ResetClicked)
            .padding(12);

        row![flash_button, reset_button]
            .spacing(10)
            .padding(10)
            .into()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let app = SimplifiedFlasher::new();
        assert!(app.selected_image.is_none());
        assert!(app.selected_device.is_none());
        assert!(!app.is_ready());
        assert_eq!(app.available_devices.len(), 3);
    }

    #[test]
    fn test_is_ready_logic() {
        let mut app = SimplifiedFlasher::new();
        assert!(!app.is_ready());

        // Add image only
        app.selected_image = Some(ImageFile {
            name: "test.iso".to_string(),
            path: PathBuf::from("/test.iso"),
            size_mb: 100.0,
        });
        assert!(!app.is_ready());

        // Add device
        app.selected_device = Some(Device {
            name: "USB".to_string(),
            size_gb: 32.0,
            path: "/dev/sdb".to_string(),
        });
        assert!(app.is_ready());
    }

    #[test]
    fn test_device_selection() {
        let mut app = SimplifiedFlasher::new();
        let device = app.available_devices[0].clone();

        let _ = app.update(Message::DeviceSelected(device.clone()));

        assert!(app.selected_device.is_some());
        assert_eq!(app.selected_device.unwrap().path, device.path);
    }

    #[test]
    fn test_reset() {
        let mut app = SimplifiedFlasher::new();

        // Set some state
        app.selected_image = Some(ImageFile {
            name: "test.iso".to_string(),
            path: PathBuf::from("/test.iso"),
            size_mb: 100.0,
        });
        app.selected_device = Some(Device {
            name: "USB".to_string(),
            size_gb: 32.0,
            path: "/dev/sdb".to_string(),
        });

        // Reset
        let _ = app.update(Message::ResetClicked);

        assert!(app.selected_image.is_none());
        assert!(app.selected_device.is_none());
    }
}

// ============================================================================
// KEY TAKEAWAYS
// ============================================================================
//
// 1. MULTI-STEP WORKFLOW:
//    - User selects an image file
//    - User selects a target device
//    - User initiates the flash operation
//
// 2. STATE MANAGEMENT:
//    - All state is explicit in the model
//    - Optional fields for selections (Option<T>)
//    - Helper methods like is_ready()
//
// 3. UI PATTERNS:
//    - Sectioned layout (image, device, actions)
//    - Conditional rendering based on state
//    - Button states (enabled/disabled)
//
// 4. THE ELM ARCHITECTURE:
//    - Model: SimplifiedFlasher struct
//    - Message: User actions and async results
//    - Update: State transitions
//    - View: Declarative UI based on state
//
// 5. REAL FLASHKRAFT DIFFERENCES:
//    - Uses rfd for actual file dialogs
//    - Uses sysinfo for real drive detection
//    - Has progress tracking during flash
//    - Includes error handling
//    - Has theme support
//    - More sophisticated validation
//
// This example shows the core patterns in a simple, understandable way!
