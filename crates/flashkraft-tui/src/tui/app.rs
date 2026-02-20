//! TUI Application State
//!
//! Defines the complete state machine for the FlashKraft terminal UI.
//! All state mutations go through [`App::handle_event`] keeping the logic
//! centralised and easy to reason about.

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::mpsc;

use crate::domain::{DriveInfo, ImageInfo};

// ---------------------------------------------------------------------------
// Flash progress events (TUI-specific, Tokio-channel-based)
// ---------------------------------------------------------------------------

/// Progress events produced by the background flash task.
#[derive(Debug, Clone)]
pub enum FlashEvent {
    /// `(progress 0.0–1.0, bytes_written, speed_mb_s)`
    Progress(f32, u64, f32),
    /// Stage / status message (e.g. "WRITING", "VERIFYING")
    Stage(String),
    /// Informational log line
    Log(String),
    /// Flash completed successfully
    Completed,
    /// Flash failed with an error message
    Failed(String),
}

// ---------------------------------------------------------------------------
// USB content entry (shown on the completion screen)
// ---------------------------------------------------------------------------

/// A single entry in the post-flash USB content listing.
#[derive(Debug, Clone)]
pub struct UsbEntry {
    pub name: String,
    pub size_bytes: u64,
    pub is_dir: bool,
    /// Nesting depth for tree-style display (0 = root)
    pub depth: usize,
}

// ---------------------------------------------------------------------------
// Application screens
// ---------------------------------------------------------------------------

/// The currently active screen / step.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum AppScreen {
    /// Step 1 — type (or paste) the path to an OS image.
    #[default]
    SelectImage,
    /// Step 2 — choose a USB drive from the detected list.
    SelectDrive,
    /// Step 2½ — pie-chart overview of the selected drive's storage.
    DriveInfo,
    /// Step 3 — confirmation dialog before writing.
    ConfirmFlash,
    /// Step 4 — flash operation in progress (tui-slider).
    Flashing,
    /// Step 5 — flash complete; show USB contents + pie-chart.
    Complete,
    /// Error screen — displayed whenever a fatal error occurs.
    Error,
}

// ---------------------------------------------------------------------------
// Input mode
// ---------------------------------------------------------------------------

/// Whether the app is currently capturing keyboard input for a text field.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Normal navigation mode.
    #[default]
    Normal,
    /// Typing into the image-path text field.
    Editing,
}

// ---------------------------------------------------------------------------
// Main application state
// ---------------------------------------------------------------------------

pub struct App {
    // ── Screen ───────────────────────────────────────────────────────────────
    /// Active screen.
    pub screen: AppScreen,

    // ── Image selection ───────────────────────────────────────────────────────
    /// Raw text the user typed in the image-path field.
    pub image_input: String,
    /// Cursor position inside `image_input`.
    pub image_cursor: usize,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Resolved & validated image, set after the user confirms.
    pub selected_image: Option<ImageInfo>,

    // ── Drive selection ───────────────────────────────────────────────────────
    /// All drives detected on the system.
    pub available_drives: Vec<DriveInfo>,
    /// Index of the currently highlighted drive in the list.
    pub drive_cursor: usize,
    /// The drive the user confirmed for flashing.
    pub selected_drive: Option<DriveInfo>,
    /// True while the background drive-detection task is running.
    pub drives_loading: bool,
    /// Channel receiving the result of the async drive-detection task.
    pub drives_rx: Option<mpsc::UnboundedReceiver<Vec<DriveInfo>>>,

    // ── Flash operation ───────────────────────────────────────────────────────
    /// Progress 0.0–1.0.
    pub flash_progress: f32,
    /// Bytes written so far.
    pub flash_bytes: u64,
    /// Current write speed in MB/s.
    pub flash_speed: f32,
    /// Human-readable stage label (e.g. "Writing…").
    pub flash_stage: String,
    /// Recent log lines from the flash helper.
    pub flash_log: Vec<String>,
    /// Shared cancellation token.
    pub cancel_token: Arc<AtomicBool>,
    /// Channel receiving [`FlashEvent`]s from the background flash task.
    pub flash_rx: Option<mpsc::UnboundedReceiver<FlashEvent>>,

    // ── Completion ────────────────────────────────────────────────────────────
    /// File/directory entries found on the USB drive after flashing.
    pub usb_contents: Vec<UsbEntry>,
    /// Scroll offset for the USB contents list.
    pub contents_scroll: usize,

    // ── Error ─────────────────────────────────────────────────────────────────
    /// Error message shown on the error screen.
    pub error_message: String,

    // ── Misc ──────────────────────────────────────────────────────────────────
    /// Ticked upward on every terminal frame; used for animations.
    pub tick_count: u64,
    /// Set to true by the event loop when the user requests quit.
    pub should_quit: bool,
}

impl App {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Self {
            screen: AppScreen::SelectImage,
            image_input: String::new(),
            image_cursor: 0,
            input_mode: InputMode::Editing, // start in edit mode for convenience
            selected_image: None,
            available_drives: Vec::new(),
            drive_cursor: 0,
            selected_drive: None,
            drives_loading: false,
            drives_rx: None,
            flash_progress: 0.0,
            flash_bytes: 0,
            flash_speed: 0.0,
            flash_stage: "Initialising…".to_string(),
            flash_log: Vec::new(),
            cancel_token: Arc::new(AtomicBool::new(false)),
            flash_rx: None,
            usb_contents: Vec::new(),
            contents_scroll: 0,
            error_message: String::new(),
            tick_count: 0,
            should_quit: false,
        }
    }

    // -----------------------------------------------------------------------
    // Channel polling — called on every tick from the main event loop
    // -----------------------------------------------------------------------

    /// Drain the drive-detection channel (if active) and apply results.
    pub fn poll_drives(&mut self) {
        // Take ownership temporarily so we can mutate `self` freely.
        let mut rx = match self.drives_rx.take() {
            Some(r) => r,
            None => return,
        };

        if let Ok(drives) = rx.try_recv() {
            self.available_drives = drives;
            self.drives_loading = false;
            self.drive_cursor = 0;
            // Don't put the receiver back — detection is one-shot.
        } else {
            // Not ready yet — put it back.
            self.drives_rx = Some(rx);
        }
    }

    /// Drain the flash-progress channel (if active) and apply results.
    pub fn poll_flash(&mut self) {
        let mut rx = match self.flash_rx.take() {
            Some(r) => r,
            None => return,
        };

        loop {
            match rx.try_recv() {
                Ok(event) => self.apply_flash_event(event),
                Err(mpsc::error::TryRecvError::Empty) => {
                    self.flash_rx = Some(rx);
                    break;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    // Task ended — channel closed; don't put receiver back.
                    break;
                }
            }
        }
    }

    fn apply_flash_event(&mut self, event: FlashEvent) {
        match event {
            FlashEvent::Progress(p, bytes, speed) => {
                self.flash_progress = p;
                self.flash_bytes = bytes;
                self.flash_speed = speed;
            }
            FlashEvent::Stage(s) => {
                self.flash_stage = s.clone();
                self.push_log(s);
            }
            FlashEvent::Log(msg) => {
                self.push_log(msg);
            }
            FlashEvent::Completed => {
                self.flash_progress = 1.0;
                self.flash_stage = "Complete!".to_string();
                self.push_log("Flash operation completed successfully.".to_string());
                self.scan_usb_contents();
                self.screen = AppScreen::Complete;
            }
            FlashEvent::Failed(err) => {
                self.error_message = err;
                self.screen = AppScreen::Error;
            }
        }
    }

    fn push_log(&mut self, msg: String) {
        const MAX_LOG: usize = 200;
        self.flash_log.push(msg);
        if self.flash_log.len() > MAX_LOG {
            self.flash_log.drain(0..self.flash_log.len() - MAX_LOG);
        }
    }

    // -----------------------------------------------------------------------
    // Navigation helpers
    // -----------------------------------------------------------------------

    /// Move drive-list cursor up.
    pub fn drive_up(&mut self) {
        if self.drive_cursor > 0 {
            self.drive_cursor -= 1;
        }
    }

    /// Move drive-list cursor down.
    pub fn drive_down(&mut self) {
        if !self.available_drives.is_empty() && self.drive_cursor < self.available_drives.len() - 1
        {
            self.drive_cursor += 1;
        }
    }

    /// Scroll USB-contents list up.
    pub fn contents_up(&mut self) {
        if self.contents_scroll > 0 {
            self.contents_scroll -= 1;
        }
    }

    /// Scroll USB-contents list down.
    pub fn contents_down(&mut self) {
        if !self.usb_contents.is_empty()
            && self.contents_scroll < self.usb_contents.len().saturating_sub(1)
        {
            self.contents_scroll += 1;
        }
    }

    // -----------------------------------------------------------------------
    // Image input helpers
    // -----------------------------------------------------------------------

    /// Insert a character at the current cursor position.
    pub fn image_insert(&mut self, c: char) {
        // Guard against inserting non-UTF-8-safe chars (shouldn't happen via
        // crossterm, but be safe).
        let byte_pos = self
            .image_input
            .char_indices()
            .nth(self.image_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.image_input.len());
        self.image_input.insert(byte_pos, c);
        self.image_cursor += 1;
    }

    /// Delete the character before the cursor (backspace).
    pub fn image_backspace(&mut self) {
        if self.image_cursor == 0 {
            return;
        }
        let byte_pos = self
            .image_input
            .char_indices()
            .nth(self.image_cursor - 1)
            .map(|(i, _)| i)
            .unwrap_or(self.image_input.len());
        self.image_input.remove(byte_pos);
        self.image_cursor -= 1;
    }

    /// Move the cursor one character to the left.
    pub fn image_cursor_left(&mut self) {
        if self.image_cursor > 0 {
            self.image_cursor -= 1;
        }
    }

    /// Move the cursor one character to the right.
    pub fn image_cursor_right(&mut self) {
        let len = self.image_input.chars().count();
        if self.image_cursor < len {
            self.image_cursor += 1;
        }
    }

    // -----------------------------------------------------------------------
    // State transitions
    // -----------------------------------------------------------------------

    /// Validate the image path and advance to the drive-selection screen.
    ///
    /// Returns `Err` with a human-readable message if the path is invalid.
    pub fn confirm_image(&mut self) -> Result<(), String> {
        let path = PathBuf::from(self.image_input.trim());
        if !path.exists() {
            return Err(format!("File not found: {}", path.display()));
        }
        if !path.is_file() {
            return Err(format!("Not a file: {}", path.display()));
        }

        let info = ImageInfo::from_path(path);
        if info.size_mb == 0.0 {
            return Err("Image file appears to be empty.".to_string());
        }

        self.selected_image = Some(info);
        self.input_mode = InputMode::Normal;
        self.screen = AppScreen::SelectDrive;
        Ok(())
    }

    /// Confirm the highlighted drive and advance to the drive-info screen.
    pub fn confirm_drive(&mut self) -> Result<(), String> {
        let drive = self
            .available_drives
            .get(self.drive_cursor)
            .cloned()
            .ok_or_else(|| "No drive selected.".to_string())?;

        if drive.is_system {
            return Err(format!(
                "{} is a system drive and cannot be used as a flash target.",
                drive.name
            ));
        }
        if drive.is_read_only {
            return Err(format!("{} is read-only.", drive.name));
        }

        self.selected_drive = Some(drive);
        self.screen = AppScreen::DriveInfo;
        Ok(())
    }

    /// Advance from drive-info to the confirmation screen.
    pub fn advance_to_confirm(&mut self) {
        self.screen = AppScreen::ConfirmFlash;
    }

    /// Begin the flash operation.
    ///
    /// Returns `Err` if state is inconsistent (image / drive not set).
    pub fn begin_flash(&mut self) -> Result<(), String> {
        let image = self
            .selected_image
            .as_ref()
            .ok_or("No image selected.")?
            .clone();
        let drive = self
            .selected_drive
            .as_ref()
            .ok_or("No drive selected.")?
            .clone();

        // Reset flash state.
        self.flash_progress = 0.0;
        self.flash_bytes = 0;
        self.flash_speed = 0.0;
        self.flash_stage = "Starting…".to_string();
        self.flash_log.clear();
        self.cancel_token = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::unbounded_channel::<FlashEvent>();
        self.flash_rx = Some(rx);

        let cancel = self.cancel_token.clone();

        // Spawn the flash task onto the Tokio runtime that owns this thread.
        tokio::spawn(crate::tui::flash_runner::run_flash(
            image.path,
            PathBuf::from(&drive.device_path),
            cancel,
            tx,
        ));

        self.screen = AppScreen::Flashing;
        Ok(())
    }

    /// Cancel an in-progress flash operation.
    pub fn cancel_flash(&mut self) {
        self.cancel_token.store(true, Ordering::SeqCst);
        self.error_message = "Flash operation cancelled by user.".to_string();
        self.screen = AppScreen::Error;
    }

    /// Full reset — go back to the first screen.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Go back one step (where meaningful).
    pub fn go_back(&mut self) {
        match self.screen {
            AppScreen::SelectDrive => {
                self.screen = AppScreen::SelectImage;
                self.input_mode = InputMode::Editing;
            }
            AppScreen::DriveInfo => {
                self.screen = AppScreen::SelectDrive;
            }
            AppScreen::ConfirmFlash => {
                self.screen = AppScreen::DriveInfo;
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // USB content scanning (post-flash)
    // -----------------------------------------------------------------------

    /// Scan the USB device's first mount point for files and populate
    /// `self.usb_contents`.  Falls back gracefully if the device is not
    /// yet mounted or the scan fails.
    fn scan_usb_contents(&mut self) {
        self.usb_contents.clear();

        let device_path = match &self.selected_drive {
            Some(d) => d.device_path.clone(),
            None => return,
        };

        // Try to find a mount point for the device (or any of its partitions).
        let mount_point = find_mount_point(&device_path);

        if let Some(mp) = mount_point {
            let root = PathBuf::from(&mp);
            let mut entries = Vec::new();
            collect_entries(&root, &root, 0, &mut entries, 3); // max 3 levels deep
            self.usb_contents = entries;
        } else {
            // Device not (yet) mounted — show a placeholder.
            self.usb_contents.push(UsbEntry {
                name: "(device not mounted — re-plug to browse)".to_string(),
                size_bytes: 0,
                is_dir: false,
                depth: 0,
            });
        }
    }

    // -----------------------------------------------------------------------
    // Convenience accessors
    // -----------------------------------------------------------------------

    /// Image size in bytes (0 if no image selected).
    pub fn image_size_bytes(&self) -> u64 {
        self.selected_image
            .as_ref()
            .map(|i| (i.size_mb * 1024.0 * 1024.0) as u64)
            .unwrap_or(0)
    }

    /// Drive capacity in bytes (0 if no drive selected).
    pub fn drive_size_bytes(&self) -> u64 {
        self.selected_drive
            .as_ref()
            .map(|d| (d.size_gb * 1024.0 * 1024.0 * 1024.0) as u64)
            .unwrap_or(0)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Parse `/proc/mounts` (Linux) or `/etc/mtab` to find a mount point for the
/// given block device or any of its partitions.
fn find_mount_point(device_path: &str) -> Option<String> {
    let device_base = std::path::Path::new(device_path)
        .file_name()?
        .to_string_lossy()
        .to_string();

    // Try /proc/mounts first (Linux), fall back to /etc/mtab.
    let mounts_content = std::fs::read_to_string("/proc/mounts")
        .or_else(|_| std::fs::read_to_string("/etc/mtab"))
        .ok()?;

    for line in mounts_content.lines() {
        let mut parts = line.split_whitespace();
        let dev = parts.next().unwrap_or("");
        let mp = parts.next().unwrap_or("");

        if mp.is_empty() || dev.is_empty() {
            continue;
        }

        let dev_base = std::path::Path::new(dev)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Match the device itself or any of its partitions (sdb, sdb1, sdb2…).
        if dev_base == device_base || dev_base.starts_with(&device_base) {
            // Skip pseudo filesystems.
            if !mp.starts_with('/') || mp == "/" {
                continue;
            }
            return Some(mp.to_string());
        }
    }

    None
}

/// Recursively collect file/directory entries up to `max_depth` levels.
fn collect_entries(
    root: &PathBuf,
    dir: &PathBuf,
    depth: usize,
    out: &mut Vec<UsbEntry>,
    max_depth: usize,
) {
    if depth > max_depth {
        return;
    }

    let read_result = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    // Collect and sort entries for a deterministic display order.
    let mut entries: Vec<_> = read_result.flatten().collect();
    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        // Directories first, then files, each sorted alphabetically.
        (!is_dir, e.file_name().to_string_lossy().to_lowercase())
    });

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files / special entries at the root level.
        if depth == 0 && name.starts_with('.') {
            continue;
        }

        let meta = path.metadata();
        let (is_dir, size) = meta
            .as_ref()
            .map(|m| (m.is_dir(), m.len()))
            .unwrap_or((false, 0));

        out.push(UsbEntry {
            name,
            size_bytes: size,
            is_dir,
            depth,
        });

        if is_dir && depth < max_depth {
            collect_entries(root, &path, depth + 1, out, max_depth);
        }
    }
}
