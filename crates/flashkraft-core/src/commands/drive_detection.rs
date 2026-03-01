//! Drive Detection — USB-aware block device enumeration
//!
//! This module enumerates removable storage devices by combining two sources:
//!
//! 1. **`nusb`** — queries the OS USB stack for every connected USB device,
//!    giving us rich metadata: vendor/product strings, serial number, USB
//!    speed, VID/PID.
//!
//! 2. **sysfs** (`/sys/block/<name>`) — walks from the kernel block device
//!    node up through its sysfs parent chain to find the USB device ancestor,
//!    mapping each `/dev/sdX` to the matching `nusb::DeviceInfo`.
//!
//! Non-USB block devices (NVMe SSDs, eMMC, internal SATA) are intentionally
//! excluded from the list — the user should never flash their system disk.
//!
//! ## Linux permission note
//!
//! `nusb::list_devices()` reads from `/sys/bus/usb/devices` which is
//! world-readable and requires no special privileges.  Writing to
//! `/dev/sdX` is handled separately by the privileged flash pipeline.

use crate::domain::drive_info::{DriveInfo, UsbInfo};
use nusb::{DeviceInfo, MaybeFuture};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Enumerate all USB mass-storage block devices currently connected.
///
/// Returns a [`Vec<DriveInfo>`] sorted by device name.  Each entry has:
/// - A human-readable name built from USB vendor/product strings + node name.
/// - The `/dev/sdX` (or `/dev/mmcblkN`) kernel block device path.
/// - Size in GB from `/sys/block/<name>/size`.
/// - `is_system` always `false` — USB devices are never the boot disk.
/// - `is_read_only` from `/sys/block/<name>/ro`.
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
    // ── 1. Enumerate USB devices via nusb ────────────────────────────────────
    let usb_devices: Vec<DeviceInfo> = match nusb::list_devices().wait() {
        Ok(iter) => iter.collect(),
        Err(e) => {
            eprintln!("[drive_detection] nusb::list_devices failed: {e}");
            return Vec::new();
        }
    };

    // Build a map sysfs_path → &DeviceInfo for fast ancestor lookups.
    // sysfs_path() is Linux-only; on other platforms the map is empty and
    // the function returns an empty drive list.
    #[allow(unused_variables)]
    let usb_by_sysfs: HashMap<PathBuf, &DeviceInfo> = usb_devices
        .iter()
        .filter_map(|d| {
            #[cfg(target_os = "linux")]
            {
                Some((d.sysfs_path().to_path_buf(), d))
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = d;
                None
            }
        })
        .collect();

    // ── 2. Walk /sys/block, keep only entries reachable from a USB node ──────
    let mut drives = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let block_root = Path::new("/sys/block");
        let Ok(entries) = std::fs::read_dir(block_root) else {
            return Vec::new();
        };

        let mounts = read_proc_mounts();

        for entry in entries.flatten() {
            let dev_name = entry.file_name().to_string_lossy().to_string();

            if should_skip_device(&dev_name) {
                continue;
            }

            let block_sysfs = block_root.join(&dev_name);

            // Canonicalize resolves the /sys/block/<name> symlink to the
            // real sysfs path, e.g.
            // /sys/devices/pci.../usb1/1-2/1-2:1.0/host0/.../block/sdb
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
    }

    // Sort by device name for stable ordering in the UI.
    drives.sort_by(|a: &DriveInfo, b: &DriveInfo| a.device_path.cmp(&b.device_path));
    drives
}

// ---------------------------------------------------------------------------
// sysfs helpers
// ---------------------------------------------------------------------------

/// Return `true` for device names that should never be offered as flash
/// targets: loop devices, RAM disks, device-mapper, zram, and NVMe/SATA/eMMC
/// internal disks.
///
/// We only want USB-connected removable storage.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn should_skip_device(name: &str) -> bool {
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
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn read_sysfs_u64(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

/// Walk `path` upward through the sysfs hierarchy until we find a directory
/// whose path exactly matches one of the keys in `usb_by_sysfs`.
///
/// The canonical sysfs path of a block device looks like:
/// ```text
/// /sys/devices/pci0000:00/…/usb1/1-2/1-2:1.0/host0/target0:0:0/0:0:0:0/block/sdb
/// ```
/// The USB device node is typically at depth -3 to -6 from the block node.
/// We walk up until we hit `/sys/devices` or `/sys` (safety limit).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn find_usb_ancestor<'a>(
    path: &Path,
    usb_by_sysfs: &'a HashMap<PathBuf, &'a DeviceInfo>,
) -> Option<&'a DeviceInfo> {
    let mut current = path.to_path_buf();

    loop {
        if let Some(dev) = usb_by_sysfs.get(&current) {
            return Some(dev);
        }

        // Stop at /sys/devices or /sys to avoid walking past the sysfs root.
        if current == Path::new("/sys/devices") || current == Path::new("/sys") {
            return None;
        }

        match current.parent() {
            Some(p) if p != current => current = p.to_path_buf(),
            _ => return None,
        }
    }
}

/// Parse `/proc/mounts` and return a map of device-name → mount-point.
///
/// Keys are bare device names such as `sdb`, not full paths.
/// Only `/dev/…` entries are included.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn read_proc_mounts() -> HashMap<String, String> {
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

        // Use just the device name (e.g. "sdb" from "/dev/sdb").
        if let Some(name) = Path::new(dev).file_name() {
            map.insert(name.to_string_lossy().to_string(), mount.to_string());
        }
    }

    map
}

/// Find the mount point for `dev_name` or any of its partitions.
///
/// Checks both the whole-disk node (`sdb`) and partition nodes
/// (`sdb1`, `sdb2`, …).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn find_mount_point(dev_name: &str, mounts: &HashMap<String, String>) -> Option<String> {
    // Check the device itself first.
    if let Some(mp) = mounts.get(dev_name) {
        return Some(mp.clone());
    }

    // Then check any partition of this device.
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

// ---------------------------------------------------------------------------
// USB metadata helpers
// ---------------------------------------------------------------------------

/// Build a [`UsbInfo`] from a `nusb::DeviceInfo`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
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
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
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
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── should_skip_device ───────────────────────────────────────────────────

    #[test]
    fn test_skip_loop_devices() {
        assert!(should_skip_device("loop0"));
        assert!(should_skip_device("loop1"));
    }

    #[test]
    fn test_skip_nvme_devices() {
        assert!(should_skip_device("nvme0n1"));
        assert!(should_skip_device("nvme1n1"));
    }

    #[test]
    fn test_skip_ram_devices() {
        assert!(should_skip_device("ram0"));
    }

    #[test]
    fn test_skip_dm_devices() {
        assert!(should_skip_device("dm-0"));
        assert!(should_skip_device("dm-1"));
    }

    #[test]
    fn test_skip_optical_drives() {
        assert!(should_skip_device("sr0"));
    }

    #[test]
    fn test_skip_empty_name() {
        assert!(should_skip_device(""));
    }

    #[test]
    fn test_allow_sata_usb_names() {
        // sda / sdb / sdc are not skipped — they may be USB drives.
        assert!(!should_skip_device("sda"));
        assert!(!should_skip_device("sdb"));
        assert!(!should_skip_device("sdc"));
    }

    // ── read_sysfs_u64 ───────────────────────────────────────────────────────

    #[test]
    fn test_read_sysfs_u64_valid() {
        let dir = std::env::temp_dir();
        let path = dir.join("fk_sysfs_u64_test.txt");
        std::fs::write(&path, "1953525168\n").unwrap();
        assert_eq!(read_sysfs_u64(&path), Some(1953525168u64));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_read_sysfs_u64_invalid() {
        let dir = std::env::temp_dir();
        let path = dir.join("fk_sysfs_invalid.txt");
        std::fs::write(&path, "not_a_number\n").unwrap();
        assert_eq!(read_sysfs_u64(&path), None);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_read_sysfs_u64_missing_file() {
        assert_eq!(read_sysfs_u64(Path::new("/nonexistent/sysfs/file")), None);
    }

    #[test]
    fn test_read_sysfs_u64_zero() {
        let dir = std::env::temp_dir();
        let path = dir.join("fk_sysfs_zero.txt");
        std::fs::write(&path, "0\n").unwrap();
        assert_eq!(read_sysfs_u64(&path), Some(0u64));
        let _ = std::fs::remove_file(path);
    }

    // ── find_mount_point ─────────────────────────────────────────────────────

    #[test]
    fn test_find_mount_point_exact_device() {
        let mut mounts = HashMap::new();
        mounts.insert("sdb".to_string(), "/media/usb".to_string());
        assert_eq!(
            find_mount_point("sdb", &mounts),
            Some("/media/usb".to_string())
        );
    }

    #[test]
    fn test_find_mount_point_partition() {
        let mut mounts = HashMap::new();
        mounts.insert("sdb1".to_string(), "/media/usb".to_string());
        assert_eq!(
            find_mount_point("sdb", &mounts),
            Some("/media/usb".to_string())
        );
    }

    #[test]
    fn test_find_mount_point_not_found() {
        let mounts = HashMap::new();
        assert_eq!(find_mount_point("sdb", &mounts), None);
    }

    #[test]
    fn test_find_mount_point_different_device_not_matched() {
        let mut mounts = HashMap::new();
        mounts.insert("sdc1".to_string(), "/media/other".to_string());
        // sdc1 must NOT match sdb
        assert_eq!(find_mount_point("sdb", &mounts), None);
    }

    // ── speed_string ─────────────────────────────────────────────────────────

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

    // ── build_display_name ───────────────────────────────────────────────────

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
        // display_label() returns "1234:abcd" → we fall back to just dev_name
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

    // ── find_usb_ancestor ────────────────────────────────────────────────────

    #[test]
    fn test_find_usb_ancestor_direct_match() {
        let mut map: HashMap<PathBuf, &DeviceInfo> = HashMap::new();

        // We cannot construct a DeviceInfo directly, but we can test the
        // "not found" path without real hardware.
        let path = PathBuf::from("/sys/devices/pci0000:00/usb1/1-2");
        // Empty map → always None.
        assert!(find_usb_ancestor(&path, &map).is_none());
        // Suppress unused mut warning.
        let _ = &mut map;
    }

    #[test]
    fn test_find_usb_ancestor_stops_at_sys_root() {
        let map: HashMap<PathBuf, &DeviceInfo> = HashMap::new();
        let path = PathBuf::from("/sys");
        assert!(find_usb_ancestor(&path, &map).is_none());
    }

    // ── read_proc_mounts ─────────────────────────────────────────────────────

    #[test]
    fn test_read_proc_mounts_does_not_panic() {
        // On any platform this should either return a populated map or an
        // empty one — it must not panic.
        let mounts = read_proc_mounts();
        let _ = mounts;
    }

    // ── load_drives_sync ─────────────────────────────────────────────────────

    #[test]
    fn test_load_drives_sync_returns_vec() {
        // On CI there may be no USB devices — we just verify it doesn't panic
        // and that every returned drive has a non-empty device_path and
        // usb_info populated.
        let drives = load_drives_sync();
        for drive in &drives {
            assert!(
                !drive.device_path.is_empty(),
                "device_path must not be empty"
            );
            assert!(
                drive.usb_info.is_some(),
                "all drives from load_drives_sync must have USB metadata"
            );
            assert!(
                drive.size_gb > 0.0,
                "size_gb must be positive: {}",
                drive.device_path
            );
        }
    }
}
