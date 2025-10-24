//! File Selection Command
//!
//! This module contains async functions for file selection dialogs.

use rfd::AsyncFileDialog;
use std::path::PathBuf;

/// Open a file dialog to select an image file
///
/// This async function shows a native file picker dialog and waits
/// for the user to select a file (or cancel).
///
/// # Returns
///
/// `Some(PathBuf)` if a file was selected, `None` if cancelled
pub async fn select_image_file() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .set_title("Select Image File")
        .add_filter(
            "Image Files",
            &["img", "iso", "dmg", "zip", "gz", "xz", "raw"],
        )
        .add_filter("All Files", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // This test just ensures the module compiles correctly
        // Actual file dialog testing requires user interaction
    }
}
