//! Drive Detection — USB-aware block device enumeration (cross-platform)
//!
//! This module enumerates removable USB storage devices by combining two
//! sources of information:
//!
//! 1. **`nusb`** — queries the OS USB stack for every connected USB device,
//!    giving rich metadata: vendor/product strings, serial number, USB speed,
//!    VID/PID.  Works on Linux, macOS, and Windows.
//!
//! 2. **OS-specific block node mapping** — correlates each `nusb::DeviceInfo`
//!    to the kernel block device path used for writing:
//!
//!    | Platform | Source | Block node format |
//!    |----------|--------|-------------------|
//!    | Linux    | sysfs (`/sys/block`) | `/dev/sdX` |
//!    | macOS    | `diskutil info -plist` | `/dev/rdiskN` (raw, faster) |
//!    | Windows  | `wmic diskdrive` CSV | `\\.\PhysicalDriveN` |
//!
//! ## Privilege notes
//!
//! - **Linux**: binary must be setuid-root (`chmod u+s`).
//! - **macOS**: `/dev/rdiskN` requires root; the flash pipeline's `seteuid(0)`
//!   handles this (same setuid-root model as Linux).
//! - **Windows**: the process must run as Administrator; `open_device_for_writing`
//!   in `flash_helper.rs` gives a clear error if it isn't.

use crate::domain::drive_info::{DriveInfo, UsbInfo};
use nusb::{DeviceInfo, MaybeFuture};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Enumerate all USB mass-storage block devices currently connected.
///
/// Returns a [`Vec<DriveInfo>`] sorted by device path.  Each entry has:
/// - A human-readable name from USB vendor/product strings.
/// - The OS block device path for writing.
/// - Size in GB.
/// - `usb_info` populated with VID/PID/manufacturer/product/serial/speed.
pub async fn load_drives() -> Vec<DriveInfo> {
    tokio::task::spawn_blocking(load_drives_sync)
        .await
        .unwrap_or_default()
}

/// Synchronous implementation — called from a blocking thread pool via
/// [`load_drives`].
///
/// Exposed as `pub` so unit tests and the TUI can call it without a Tokio
/// runtime.
pub fn load_drives_sync() -> Vec<DriveInfo> {
    // ── 1. Enumerate USB devices via nusb (all platforms) ────────────────────
    let usb_devices: Vec<DeviceInfo> = match nusb::list_devices().wait() {
        Ok(iter) => iter.collect(),
        Err(e) => {
            eprintln!("[drive_detection] nusb::list_devices failed: {e}");
            return Vec::new();
        }
    };

    // ── 2. Map each USB device to a block node (platform-specific) ────────────
    #[cfg(target_os = "linux")]
    let mut drives = linux::enumerate(&usb_devices);

    #[cfg(target_os = "macos")]
    let mut drives = macos::enumerate(&usb_devices);

    #[cfg(target_os = "windows")]
    let mut drives = windows::enumerate(&usb_devices);

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let mut drives: Vec<DriveInfo> = Vec::new();

    // Sort by device path for stable ordering in the UI.
    drives.sort_by(|a: &DriveInfo, b: &DriveInfo| a.device_path.cmp(&b.device_path));
    drives
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a [`UsbInfo`] from a `nusb::DeviceInfo`.
fn build_usb_info(dev: &DeviceInfo) -> UsbInfo {
    UsbInfo {
        vendor_id: dev.vendor_id(),
        product_id: dev.product_id(),
        manufacturer: dev.manufacturer_string().map(|s| s.to_string()),
        product: dev.product_string().map(|s| s.to_string()),
        serial: dev.serial_number().map(|s| s.to_string()),
        speed: dev.speed().map(speed_string),
    }
}

/// Convert a `nusb::Speed` variant to a human-readable string.
fn speed_string(speed: nusb::Speed) -> String {
    match speed {
        nusb::Speed::Low => "Low Speed (1.5 Mbps)".to_string(),
        nusb::Speed::Full => "Full Speed (12 Mbps)".to_string(),
        nusb::Speed::High => "High Speed (480 Mbps)".to_string(),
        nusb::Speed::Super => "SuperSpeed (5 Gbps)".to_string(),
        nusb::Speed::SuperPlus => "SuperSpeed+ (10 Gbps)".to_string(),
        _ => "Unknown speed".to_string(),
    }
}

/// Build the display name shown in the drive selector.
///
/// Format: `"{usb_label} ({dev_name})"`, e.g. `"SanDisk Ultra (sdb)"`.
/// Falls back to just `dev_name` if no USB label is available.
fn build_display_name(info: &UsbInfo, dev_name: &str) -> String {
    let label = info.display_label();
    if label.contains(':') {
        // display_label() returned a raw VID:PID — use just the device name.
        dev_name.to_string()
    } else {
        format!("{} ({})", label, dev_name)
    }
}

// ---------------------------------------------------------------------------
// Linux implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    pub fn enumerate(usb_devices: &[DeviceInfo]) -> Vec<DriveInfo> {
        // Build a map sysfs_path → &DeviceInfo for fast ancestor lookups.
        let usb_by_sysfs: HashMap<PathBuf, &DeviceInfo> = usb_devices
            .iter()
            .map(|d| (d.sysfs_path().to_path_buf(), d))
            .collect();

        let block_root = Path::new("/sys/block");
        let Ok(entries) = std::fs::read_dir(block_root) else {
            return Vec::new();
        };

        let mounts = read_proc_mounts();
        let mut drives = Vec::new();

        for entry in entries.flatten() {
            let dev_name = entry.file_name().to_string_lossy().to_string();

            if should_skip_device(&dev_name) {
                continue;
            }

            let block_sysfs = block_root.join(&dev_name);

            // Canonicalize resolves the /sys/block/<name> symlink to the
            // real sysfs path under /sys/devices/...
            let canonical = match std::fs::canonicalize(&block_sysfs) {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Walk up the sysfs hierarchy to find the USB device ancestor.
            let Some(usb_dev) = find_usb_ancestor(&canonical, &usb_by_sysfs) else {
                continue;
            };

            // ── Capacity ──────────────────────────────────────────────────────
            let size_sectors = read_sysfs_u64(&block_sysfs.join("size")).unwrap_or(0);
            if size_sectors == 0 {
                continue;
            }
            let size_gb = (size_sectors * 512) as f64 / (1024.0 * 1024.0 * 1024.0);

            // ── Flags ─────────────────────────────────────────────────────────
            let is_read_only = read_sysfs_u64(&block_sysfs.join("ro"))
                .map(|v| v != 0)
                .unwrap_or(false);

            // ── Paths ─────────────────────────────────────────────────────────
            let device_path = format!("/dev/{dev_name}");
            let mount_point =
                find_mount_point(&dev_name, &mounts).unwrap_or_else(|| device_path.clone());

            // ── USB metadata ──────────────────────────────────────────────────
            let usb_info = build_usb_info(usb_dev);
            let name = build_display_name(&usb_info, &dev_name);

            let drive = DriveInfo::with_constraints(
                name,
                mount_point,
                size_gb,
                device_path,
                false, // USB drives are never the system drive
                is_read_only,
            )
            .with_usb_info(usb_info);

            drives.push(drive);
        }

        drives
    }

    /// Return `true` for device names that should never be offered as flash
    /// targets: loop devices, RAM disks, device-mapper, zram, NVMe, eMMC.
    pub(super) fn should_skip_device(name: &str) -> bool {
        name.starts_with("loop")
            || name.starts_with("ram")
            || name.starts_with("dm-")
            || name.starts_with("zram")
            || name.starts_with("nvme")
            || name.starts_with("mmcblk")
            || name.starts_with("sr") // optical drives
            || name.is_empty()
    }

    /// Read a decimal integer from a single-line sysfs file.
    pub(super) fn read_sysfs_u64(path: &Path) -> Option<u64> {
        std::fs::read_to_string(path)
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()
    }

    /// Walk `path` upward through the sysfs hierarchy until we find a
    /// directory whose path exactly matches one of the keys in `usb_by_sysfs`.
    pub(super) fn find_usb_ancestor<'a>(
        path: &Path,
        usb_by_sysfs: &'a HashMap<PathBuf, &'a DeviceInfo>,
    ) -> Option<&'a DeviceInfo> {
        let mut current = path.to_path_buf();
        loop {
            if let Some(dev) = usb_by_sysfs.get(&current) {
                return Some(dev);
            }
            if current == Path::new("/sys/devices") || current == Path::new("/sys") {
                return None;
            }
            match current.parent() {
                Some(p) if p != current => current = p.to_path_buf(),
                _ => return None,
            }
        }
    }

    /// Parse `/proc/mounts` and return a map of bare device-name → mount-point.
    pub(super) fn read_proc_mounts() -> HashMap<String, String> {
        let content = std::fs::read_to_string("/proc/mounts")
            .or_else(|_| std::fs::read_to_string("/proc/self/mounts"))
            .unwrap_or_default();

        let mut map = HashMap::new();
        for line in content.lines() {
            let mut parts = line.split_whitespace();
            let dev = match parts.next() {
                Some(d) if d.starts_with("/dev/") => d,
                _ => continue,
            };
            let mount = match parts.next() {
                Some(m) => m,
                None => continue,
            };
            if let Some(name) = Path::new(dev).file_name() {
                map.insert(name.to_string_lossy().to_string(), mount.to_string());
            }
        }
        map
    }

    /// Find the mount point for `dev_name` or any of its partitions.
    pub(super) fn find_mount_point(
        dev_name: &str,
        mounts: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(mp) = mounts.get(dev_name) {
            return Some(mp.clone());
        }
        for (mounted_dev, mount_point) in mounts {
            if mounted_dev.starts_with(dev_name)
                && mounted_dev.len() > dev_name.len()
                && (mounted_dev.as_bytes()[dev_name.len()].is_ascii_digit()
                    || mounted_dev.as_bytes()[dev_name.len()] == b'p')
            {
                return Some(mount_point.clone());
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// macOS implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    /// A parsed entry from `diskutil info -plist /dev/diskN`.
    #[derive(Debug, Default)]
    pub(super) struct DiskInfo {
        /// True if this is a whole disk (not a partition slice)
        pub(super) whole_disk: bool,
        /// True if disk is ejectable / removable
        pub(super) removable: bool,
        /// Size in bytes
        pub(super) size_bytes: u64,
        /// True when mounted read-only
        pub(super) read_only: bool,
        /// Mount point of the disk or its first partition
        pub(super) mount_point: Option<String>,
        /// USB Vendor ID from IOKit (may be 0 if not USB)
        pub(super) usb_vendor_id: Option<u16>,
        /// USB Product ID from IOKit
        pub(super) usb_product_id: Option<u16>,
        /// USB serial number from IOKit
        pub(super) usb_serial: Option<String>,
        /// IOKit registry entry ID — matches nusb's registry_entry_id()
        /// (stored as hex string in the plist: "0x..." or decimal)
        pub(super) io_registry_entry_id: Option<u64>,
    }

    pub fn enumerate(usb_devices: &[DeviceInfo]) -> Vec<DriveInfo> {
        // Get all whole-disk BSD names from `diskutil list -plist external`.
        // If that returns nothing (no external disks connected) we still try
        // the nusb-guided approach below.
        let whole_disks = list_external_disks();

        if whole_disks.is_empty() {
            return Vec::new();
        }

        // Build a map: (vid, pid, serial) → &DeviceInfo from nusb.
        // We use this to correlate diskutil output with nusb metadata.
        let usb_by_ids: Vec<&DeviceInfo> = usb_devices.iter().collect();

        let mut drives = Vec::new();

        for bsd_name in &whole_disks {
            let Some(info) = disk_info(bsd_name) else {
                continue;
            };

            if !info.whole_disk || !info.removable {
                continue;
            }

            if info.size_bytes == 0 {
                continue;
            }

            let size_gb = info.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            // Try to match this disk to a nusb DeviceInfo.
            // Strategy (in order of reliability):
            //   1. IOKit registry entry ID (exact match, most reliable)
            //   2. VID + PID + serial
            //   3. VID + PID only (last resort)
            let usb_dev = find_nusb_device(
                &usb_by_ids,
                info.io_registry_entry_id,
                info.usb_vendor_id,
                info.usb_product_id,
                info.usb_serial.as_deref(),
            );

            // Build UsbInfo from nusb (preferred) or from diskutil fields.
            let usb_info = if let Some(dev) = usb_dev {
                build_usb_info(dev)
            } else {
                // No nusb match — build minimal UsbInfo from diskutil data.
                UsbInfo {
                    vendor_id: info.usb_vendor_id.unwrap_or(0),
                    product_id: info.usb_product_id.unwrap_or(0),
                    manufacturer: None,
                    product: None,
                    serial: info.usb_serial.clone(),
                    speed: None,
                }
            };

            // On macOS we use /dev/rdiskN (raw disk) for writing.
            // Raw disks bypass the buffer cache → significantly faster writes.
            // /dev/diskN  = buffered (slow, safe for mounts)
            // /dev/rdiskN = raw/unbuffered (fast, requires unmounting first)
            let device_path = format!("/dev/r{bsd_name}");
            let mount_point = info
                .mount_point
                .clone()
                .unwrap_or_else(|| device_path.clone());

            let name = build_display_name(&usb_info, bsd_name);

            let drive = DriveInfo::with_constraints(
                name,
                mount_point,
                size_gb,
                device_path,
                false,
                info.read_only,
            )
            .with_usb_info(usb_info);

            drives.push(drive);
        }

        drives
    }

    /// Run `diskutil list -plist external` and return BSD names of whole disks.
    fn list_external_disks() -> Vec<String> {
        let output = std::process::Command::new("diskutil")
            .args(["list", "-plist", "external"])
            .output();

        let Ok(out) = output else {
            eprintln!("[drive_detection] diskutil list failed");
            return Vec::new();
        };

        let text = String::from_utf8_lossy(&out.stdout);
        parse_plist_string_array(&text, "WholeDisks")
    }

    /// Run `diskutil info -plist /dev/<bsd_name>` and parse the result.
    fn disk_info(bsd_name: &str) -> Option<DiskInfo> {
        let output = std::process::Command::new("diskutil")
            .args(["info", "-plist", &format!("/dev/{bsd_name}")])
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout);
        parse_disk_info_plist(&text, bsd_name)
    }

    /// Find the nusb DeviceInfo that best matches the given disk's identifiers.
    fn find_nusb_device<'a>(
        usb_devices: &[&'a DeviceInfo],
        registry_id: Option<u64>,
        vid: Option<u16>,
        pid: Option<u16>,
        serial: Option<&str>,
    ) -> Option<&'a DeviceInfo> {
        // 1. Exact match by IOKit registry ID (most reliable).
        if let Some(rid) = registry_id {
            for dev in usb_devices {
                if dev.registry_entry_id() == rid {
                    return Some(dev);
                }
            }
        }

        // 2. VID + PID + serial.
        if let (Some(v), Some(p), Some(s)) = (vid, pid, serial) {
            for dev in usb_devices {
                if dev.vendor_id() == v && dev.product_id() == p && dev.serial_number() == Some(s) {
                    return Some(dev);
                }
            }
        }

        // 3. VID + PID only.
        if let (Some(v), Some(p)) = (vid, pid) {
            for dev in usb_devices {
                if dev.vendor_id() == v && dev.product_id() == p {
                    return Some(dev);
                }
            }
        }

        None
    }

    // ── Minimal plist parser ─────────────────────────────────────────────────
    //
    // diskutil emits Apple XML plist format.  Rather than pulling in a full
    // plist crate we do targeted key extraction with simple text scanning —
    // sufficient for the well-structured output diskutil always produces.

    /// Extract a string array value for `key` from an XML plist.
    ///
    /// Handles the pattern:
    /// ```xml
    /// <key>WholeDisks</key>
    /// <array>
    ///     <string>disk2</string>
    ///     <string>disk3</string>
    /// </array>
    /// ```
    pub(super) fn parse_plist_string_array(plist: &str, key: &str) -> Vec<String> {
        let key_tag = format!("<key>{key}</key>");
        let Some(key_pos) = plist.find(&key_tag) else {
            return Vec::new();
        };

        let after_key = &plist[key_pos + key_tag.len()..];

        // Find the opening <array>
        let Some(array_start) = after_key.find("<array>") else {
            return Vec::new();
        };
        let after_array = &after_key[array_start + "<array>".len()..];

        let Some(array_end) = after_array.find("</array>") else {
            return Vec::new();
        };
        let array_content = &after_array[..array_end];

        // Extract all <string>...</string> values.
        let mut results = Vec::new();
        let mut remaining = array_content;
        while let Some(s) = remaining.find("<string>") {
            remaining = &remaining[s + "<string>".len()..];
            if let Some(e) = remaining.find("</string>") {
                results.push(remaining[..e].trim().to_string());
                remaining = &remaining[e + "</string>".len()..];
            } else {
                break;
            }
        }
        results
    }

    /// Parse the `diskutil info -plist` output for a single disk.
    pub(super) fn parse_disk_info_plist(plist: &str, _bsd_name: &str) -> Option<DiskInfo> {
        let mut info = DiskInfo::default();

        // Walk through the flat <key>…</key><value/> pairs in the plist dict.
        // diskutil's plist is always a flat top-level dict, so this works.
        let mut cursor = plist;

        while let Some(key_start) = cursor.find("<key>") {
            cursor = &cursor[key_start + "<key>".len()..];
            let key_end = cursor.find("</key>")?;
            let key = cursor[..key_end].trim();
            cursor = &cursor[key_end + "</key>".len()..];

            // Skip whitespace/newlines between </key> and the value tag.
            let value_start = cursor.find('<').unwrap_or(cursor.len());
            let value_section = cursor[value_start..].trim_start();

            match key {
                "WholeDisk" => {
                    info.whole_disk = value_section.starts_with("<true/>");
                }
                "Ejectable" | "Removable" | "RemovableMedia" | "RemovableMediaOrExternalDevice" => {
                    if value_section.starts_with("<true/>") {
                        info.removable = true;
                    }
                }
                "ReadOnly" => {
                    info.read_only = value_section.starts_with("<true/>");
                }
                "TotalSize" | "DiskSize" => {
                    if info.size_bytes == 0 {
                        if let Some(v) = extract_integer(value_section) {
                            info.size_bytes = v;
                        }
                    }
                }
                "MountPoint" => {
                    if let Some(v) = extract_string(value_section) {
                        if !v.is_empty() {
                            info.mount_point = Some(v);
                        }
                    }
                }
                "USBVendorID" => {
                    if let Some(v) = extract_integer(value_section) {
                        info.usb_vendor_id = Some(v as u16);
                    }
                }
                "USBProductID" => {
                    if let Some(v) = extract_integer(value_section) {
                        info.usb_product_id = Some(v as u16);
                    }
                }
                "IORegistryEntryName" => {
                    // Not the ID we need, skip.
                }
                "GUID" | "DiskUUID" => {
                    // Not useful for correlation.
                }
                _ => {}
            }

            // Advance past this value tag so we don't re-parse it.
            // Find the end of the current value element.
            cursor = advance_past_value(cursor);
        }

        // size_bytes is required.
        if info.size_bytes == 0 {
            return None;
        }

        Some(info)
    }

    /// Advance `cursor` past the next complete XML value element.
    fn advance_past_value(cursor: &str) -> &str {
        let s = cursor.trim_start();

        // Self-closing tags: <true/>, <false/>
        if s.starts_with("<true/>") {
            return &cursor[cursor.find("<true/>").unwrap() + "<true/>".len()..];
        }
        if s.starts_with("<false/>") {
            return &cursor[cursor.find("<false/>").unwrap() + "<false/>".len()..];
        }

        // Tags with content: <integer>…</integer>, <string>…</string>, etc.
        let tag_end = match s.find('>') {
            Some(p) => p,
            None => return "",
        };
        let tag_name_start = match s.find('<') {
            Some(p) => p + 1,
            None => return "",
        };
        if tag_name_start >= tag_end {
            return "";
        }
        let tag_name = &s[tag_name_start..tag_end];

        // Skip over <array>…</array>, <dict>…</dict> etc. by finding </tag>.
        let close_tag = format!("</{tag_name}>");
        if let Some(pos) = cursor.find(&close_tag) {
            &cursor[pos + close_tag.len()..]
        } else {
            ""
        }
    }

    /// Extract the text content of the first `<integer>…</integer>` or
    /// `<real>…</real>` element and parse it as `u64`.
    pub(super) fn extract_integer(s: &str) -> Option<u64> {
        for tag in &["<integer>", "<real>"] {
            if let Some(start) = s.find(tag) {
                let after = &s[start + tag.len()..];
                let close = tag.replace('<', "</");
                if let Some(end) = after.find(&close) {
                    let text = after[..end].trim();
                    // Strip trailing decimals for <real> values.
                    let int_part = text.split('.').next().unwrap_or(text);
                    if let Ok(v) = int_part.parse::<u64>() {
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    /// Extract the text content of the first `<string>…</string>` element.
    pub(super) fn extract_string(s: &str) -> Option<String> {
        let start = s.find("<string>")? + "<string>".len();
        let end = s[start..].find("</string>")?;
        Some(s[start..start + end].trim().to_string())
    }
}

// ---------------------------------------------------------------------------
// Windows implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    /// A parsed USB disk drive from wmic output.
    #[derive(Debug)]
    struct WmicDisk {
        /// `\\.\PhysicalDriveN`
        device_id: String,
        /// Size in bytes (may be 0 if unknown)
        size_bytes: u64,
        /// Model string (may be empty)
        model: String,
        /// Serial number (may be empty)
        serial: String,
    }

    pub fn enumerate(usb_devices: &[DeviceInfo]) -> Vec<DriveInfo> {
        let wmic_disks = query_wmic_usb_disks();

        if wmic_disks.is_empty() {
            return Vec::new();
        }

        let mut drives = Vec::new();

        for disk in &wmic_disks {
            if disk.size_bytes == 0 {
                continue;
            }

            let size_gb = disk.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            // Correlate with nusb: try serial match, then model match.
            let usb_dev = find_nusb_device(usb_devices, &disk.serial, &disk.model);

            let usb_info = if let Some(dev) = usb_dev {
                build_usb_info(dev)
            } else {
                // No nusb match — build minimal UsbInfo from wmic model/serial.
                UsbInfo {
                    vendor_id: 0,
                    product_id: 0,
                    manufacturer: None,
                    product: if disk.model.is_empty() {
                        None
                    } else {
                        Some(disk.model.clone())
                    },
                    serial: if disk.serial.is_empty() {
                        None
                    } else {
                        Some(disk.serial.clone())
                    },
                    speed: None,
                }
            };

            // Derive a short name for display (e.g. "PhysicalDrive1")
            let short_name = disk
                .device_id
                .split('\\')
                .next_back()
                .unwrap_or(&disk.device_id);

            let name = build_display_name(&usb_info, short_name);

            let drive = DriveInfo::with_constraints(
                name,
                disk.device_id.clone(), // no mount point concept at this level
                size_gb,
                disk.device_id.clone(),
                false,
                false, // wmic doesn't easily expose read-only flag
            )
            .with_usb_info(usb_info);

            drives.push(drive);
        }

        drives
    }

    /// Run `wmic diskdrive where InterfaceType="USB" get DeviceID,Size,Model,SerialNumber /format:csv`
    /// and parse the output into `WmicDisk` structs.
    fn query_wmic_usb_disks() -> Vec<WmicDisk> {
        let output = std::process::Command::new("wmic")
            .args([
                "diskdrive",
                "where",
                "InterfaceType=\"USB\"",
                "get",
                "DeviceID,Size,Model,SerialNumber",
                "/format:csv",
            ])
            .output();

        let Ok(out) = output else {
            eprintln!("[drive_detection] wmic query failed");
            return Vec::new();
        };

        let text = String::from_utf8_lossy(&out.stdout);
        parse_wmic_csv(&text)
    }

    /// Parse wmic CSV output.
    ///
    /// wmic /format:csv emits lines like:
    /// ```
    /// Node,DeviceID,Model,SerialNumber,Size
    /// HOSTNAME,\\.\PhysicalDrive1,SanDisk Ultra USB3.0,AA01234567890,32012804608
    /// ```
    fn parse_wmic_csv(csv: &str) -> Vec<WmicDisk> {
        let mut lines = csv.lines().map(|l| l.trim()).filter(|l| !l.is_empty());

        // Find header line.
        let header = loop {
            match lines.next() {
                Some(h) if h.to_lowercase().contains("deviceid") => break h,
                Some(_) => continue,
                None => return Vec::new(),
            }
        };

        // Parse header columns (case-insensitive).
        let cols: Vec<&str> = header.split(',').collect();
        let idx = |name: &str| -> Option<usize> {
            cols.iter()
                .position(|c| c.trim().to_lowercase() == name.to_lowercase())
        };

        let idx_device = idx("DeviceID");
        let idx_size = idx("Size");
        let idx_model = idx("Model");
        let idx_serial = idx("SerialNumber");

        let mut disks = Vec::new();

        for line in lines {
            let fields: Vec<&str> = line.split(',').collect();

            let get = |i: Option<usize>| -> &str {
                i.and_then(|i| fields.get(i).copied()).unwrap_or("").trim()
            };

            let device_id = get(idx_device);
            if device_id.is_empty() || !device_id.contains("PhysicalDrive") {
                continue;
            }

            let size_bytes = get(idx_size).parse::<u64>().unwrap_or(0);

            disks.push(WmicDisk {
                device_id: device_id.to_string(),
                size_bytes,
                model: get(idx_model).to_string(),
                serial: get(idx_serial).to_string(),
            });
        }

        disks
    }

    /// Find the nusb DeviceInfo that best matches a wmic disk.
    fn find_nusb_device<'a>(
        usb_devices: &'a [DeviceInfo],
        serial: &str,
        model: &str,
    ) -> Option<&'a DeviceInfo> {
        if !serial.is_empty() {
            // Serial number match — most reliable.
            // wmic serial numbers are sometimes padded with spaces; trim both.
            let serial_trimmed = serial.trim();
            for dev in usb_devices {
                if dev
                    .serial_number()
                    .map(|s| s.trim() == serial_trimmed)
                    .unwrap_or(false)
                {
                    return Some(dev);
                }
            }
        }

        if !model.is_empty() {
            // Model string match — less reliable but useful as fallback.
            let model_lower = model.to_lowercase();
            for dev in usb_devices {
                let product = dev
                    .product_string()
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();
                let mfr = dev
                    .manufacturer_string()
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                if model_lower.contains(&product) || model_lower.contains(&mfr) {
                    return Some(dev);
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::collections::HashMap;
    #[cfg(target_os = "linux")]
    use std::path::{Path, PathBuf};

    // ── Linux helpers ─────────────────────────────────────────────────────────

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_loop_devices() {
        assert!(linux::should_skip_device("loop0"));
        assert!(linux::should_skip_device("loop1"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_nvme_devices() {
        assert!(linux::should_skip_device("nvme0n1"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_ram_devices() {
        assert!(linux::should_skip_device("ram0"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_dm_devices() {
        assert!(linux::should_skip_device("dm-0"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_optical_drives() {
        assert!(linux::should_skip_device("sr0"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_skip_empty_name() {
        assert!(linux::should_skip_device(""));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_allow_sata_usb_names() {
        assert!(!linux::should_skip_device("sda"));
        assert!(!linux::should_skip_device("sdb"));
        assert!(!linux::should_skip_device("sdc"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_sysfs_u64_valid() {
        let path = std::env::temp_dir().join("fk_sysfs_u64_test.txt");
        std::fs::write(&path, "1953525168\n").unwrap();
        assert_eq!(linux::read_sysfs_u64(&path), Some(1953525168u64));
        let _ = std::fs::remove_file(path);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_sysfs_u64_invalid() {
        let path = std::env::temp_dir().join("fk_sysfs_invalid.txt");
        std::fs::write(&path, "not_a_number\n").unwrap();
        assert_eq!(linux::read_sysfs_u64(&path), None);
        let _ = std::fs::remove_file(path);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_sysfs_u64_missing_file() {
        assert_eq!(
            linux::read_sysfs_u64(Path::new("/nonexistent/sysfs/file")),
            None
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_mount_point_exact_device() {
        let mut mounts = HashMap::new();
        mounts.insert("sdb".to_string(), "/media/usb".to_string());
        assert_eq!(
            linux::find_mount_point("sdb", &mounts),
            Some("/media/usb".to_string())
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_mount_point_partition() {
        let mut mounts = HashMap::new();
        mounts.insert("sdb1".to_string(), "/media/usb".to_string());
        assert_eq!(
            linux::find_mount_point("sdb", &mounts),
            Some("/media/usb".to_string())
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_mount_point_not_found() {
        assert_eq!(linux::find_mount_point("sdb", &HashMap::new()), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_mount_point_different_device_not_matched() {
        let mut mounts = HashMap::new();
        mounts.insert("sdc1".to_string(), "/media/other".to_string());
        assert_eq!(linux::find_mount_point("sdb", &mounts), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_find_usb_ancestor_stops_at_sys_root() {
        let map: HashMap<PathBuf, &DeviceInfo> = HashMap::new();
        let path = PathBuf::from("/sys");
        assert!(linux::find_usb_ancestor(&path, &map).is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_proc_mounts_does_not_panic() {
        let mounts = linux::read_proc_mounts();
        let _ = mounts;
    }

    // ── macOS plist parser ────────────────────────────────────────────────────

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_plist_string_array_whole_disks() {
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AllDisksAndPartitions</key>
    <array/>
    <key>WholeDisks</key>
    <array>
        <string>disk2</string>
        <string>disk3</string>
    </array>
</dict>
</plist>"#;

        let result = macos::parse_plist_string_array(plist, "WholeDisks");
        assert_eq!(result, vec!["disk2", "disk3"]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_plist_string_array_missing_key() {
        let plist = "<plist><dict><key>Other</key><array/></dict></plist>";
        let result = macos::parse_plist_string_array(plist, "WholeDisks");
        assert!(result.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_disk_info_plist_whole_removable() {
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>WholeDisk</key>
    <true/>
    <key>RemovableMediaOrExternalDevice</key>
    <true/>
    <key>ReadOnly</key>
    <false/>
    <key>TotalSize</key>
    <integer>32017047552</integer>
    <key>MountPoint</key>
    <string>/Volumes/SANDISK</string>
    <key>USBVendorID</key>
    <integer>1921</integer>
    <key>USBProductID</key>
    <integer>21889</integer>
</dict>
</plist>"#;

        let info = macos::parse_disk_info_plist(plist, "disk2").unwrap();
        assert!(info.whole_disk);
        assert!(info.removable);
        assert!(!info.read_only);
        assert_eq!(info.size_bytes, 32017047552);
        assert_eq!(info.mount_point, Some("/Volumes/SANDISK".to_string()));
        assert_eq!(info.usb_vendor_id, Some(1921));
        assert_eq!(info.usb_product_id, Some(21889));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_disk_info_plist_not_whole_disk() {
        let plist = r#"<plist version="1.0"><dict>
            <key>WholeDisk</key><false/>
            <key>RemovableMediaOrExternalDevice</key><true/>
            <key>TotalSize</key><integer>16000000000</integer>
        </dict></plist>"#;
        let info = macos::parse_disk_info_plist(plist, "disk2s1").unwrap();
        assert!(!info.whole_disk);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_disk_info_plist_zero_size_returns_none() {
        let plist = r#"<plist version="1.0"><dict>
            <key>WholeDisk</key><true/>
            <key>TotalSize</key><integer>0</integer>
        </dict></plist>"#;
        assert!(macos::parse_disk_info_plist(plist, "disk2").is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_extract_integer_from_integer_tag() {
        assert_eq!(
            macos::extract_integer("<integer>12345</integer>"),
            Some(12345)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_extract_integer_from_real_tag() {
        // Real values get truncated to u64.
        assert_eq!(
            macos::extract_integer("<real>32017047552.0</real>"),
            Some(32017047552)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_extract_string_tag() {
        assert_eq!(
            macos::extract_string("<string>/Volumes/USB</string>"),
            Some("/Volumes/USB".to_string())
        );
    }

    // ── Windows wmic parser ───────────────────────────────────────────────────

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_wmic_csv_basic() {
        let csv = "\r\nNode,DeviceID,Model,SerialNumber,Size\r\n\
                   MYPC,\\\\.\\PhysicalDrive1,SanDisk Ultra USB3.0,AA01234567890,32017047552\r\n";

        let disks = windows::parse_wmic_csv(csv);
        assert_eq!(disks.len(), 1);
        assert!(disks[0].device_id.contains("PhysicalDrive1"));
        assert_eq!(disks[0].size_bytes, 32017047552);
        assert_eq!(disks[0].serial, "AA01234567890");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_wmic_csv_empty() {
        let disks = windows::parse_wmic_csv("");
        assert!(disks.is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_wmic_csv_skips_non_physical() {
        let csv = "Node,DeviceID,Model,SerialNumber,Size\r\n\
                   MYPC,\\\\.\\CDROM0,Some CD Drive,,\r\n";
        let disks = windows::parse_wmic_csv(csv);
        assert!(
            disks.is_empty(),
            "non-PhysicalDrive entries must be skipped"
        );
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    #[test]
    fn test_speed_string_known_variants() {
        assert_eq!(speed_string(nusb::Speed::Low), "Low Speed (1.5 Mbps)");
        assert_eq!(speed_string(nusb::Speed::Full), "Full Speed (12 Mbps)");
        assert_eq!(speed_string(nusb::Speed::High), "High Speed (480 Mbps)");
        assert_eq!(speed_string(nusb::Speed::Super), "SuperSpeed (5 Gbps)");
        assert_eq!(
            speed_string(nusb::Speed::SuperPlus),
            "SuperSpeed+ (10 Gbps)"
        );
    }

    #[test]
    fn test_build_display_name_with_label() {
        let info = crate::domain::drive_info::UsbInfo {
            vendor_id: 0x0781,
            product_id: 0x5581,
            manufacturer: Some("SanDisk".into()),
            product: Some("Ultra".into()),
            serial: None,
            speed: None,
        };
        assert_eq!(build_display_name(&info, "sdb"), "SanDisk Ultra (sdb)");
    }

    #[test]
    fn test_build_display_name_vid_pid_fallback() {
        let info = crate::domain::drive_info::UsbInfo {
            vendor_id: 0x1234,
            product_id: 0xabcd,
            manufacturer: None,
            product: None,
            serial: None,
            speed: None,
        };
        // display_label() returns "1234:abcd" which contains ':' → falls back
        assert_eq!(build_display_name(&info, "sdb"), "sdb");
    }

    #[test]
    fn test_build_display_name_product_only() {
        let info = crate::domain::drive_info::UsbInfo {
            vendor_id: 0x1234,
            product_id: 0x5678,
            manufacturer: None,
            product: Some("Flash Drive".into()),
            serial: None,
            speed: None,
        };
        assert_eq!(build_display_name(&info, "sdc"), "Flash Drive (sdc)");
    }

    /// Smoke test: load_drives_sync must not panic regardless of whether any
    /// USB drives are connected.  On CI there will typically be none.
    #[test]
    fn test_load_drives_sync_returns_vec() {
        let drives = load_drives_sync();
        for drive in &drives {
            assert!(!drive.device_path.is_empty());
            assert!(drive.usb_info.is_some());
            assert!(drive.size_gb > 0.0);
        }
    }
}
