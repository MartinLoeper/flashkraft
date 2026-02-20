//! Drive Detection Command
//!
//! This module contains async functions for detecting storage drives on the system.

use std::collections::HashMap;
use std::path::Path;

use sysinfo::Disks;

use crate::domain::DriveInfo;

/// Load all available drives from the system
///
/// This async function queries the system for all block devices.
/// It checks /sys/block for all devices and combines with mount info.
///
/// # Returns
///
/// A vector of `DriveInfo` representing detected drives
pub async fn load_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    // First, get mounted filesystems
    let disks = Disks::new_with_refreshed_list();
    let mut mounted: HashMap<String, (String, u64)> = HashMap::new();

    for disk in disks.list() {
        let name = disk.name().to_string_lossy().to_string();
        let mount_point = disk.mount_point().to_string_lossy().to_string();
        let size = disk.total_space();

        if !mount_point.is_empty() {
            // Extract device name from mount point or name
            let device_name = if name.starts_with('/') {
                Path::new(&name)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&name)
                    .to_string()
            } else {
                name
            };

            mounted.insert(device_name, (mount_point, size));
        }
    }

    // Now scan /sys/block for all block devices
    if let Ok(entries) = std::fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();

            // Skip loop devices, ram disks, and other virtual devices
            if device_name.starts_with("loop")
                || device_name.starts_with("ram")
                || device_name.starts_with("dm-")
                || device_name.starts_with("zram")
            {
                continue;
            }

            // Check if this is a system drive by checking if it's mounted at critical paths
            let is_system = check_if_system_drive(&device_name);

            // Check if drive is read-only
            let is_read_only = check_if_read_only(&device_name);

            // Read device size from sysfs
            let size_path = format!("/sys/block/{}/size", device_name);
            let size_sectors = std::fs::read_to_string(&size_path)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);

            // Convert sectors to GB (sector size is 512 bytes)
            let size_gb = (size_sectors * 512) as f64 / (1024.0 * 1024.0 * 1024.0);

            // Include all devices with size > 0, even very small ones (like 64KB test devices)
            if size_sectors > 0 {
                // Try to get device model name
                let model_path = format!("/sys/block/{}/device/model", device_name);
                let vendor_path = format!("/sys/block/{}/device/vendor", device_name);

                let model = std::fs::read_to_string(&model_path)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                let vendor = std::fs::read_to_string(&vendor_path)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                // Build display name with model info if available
                let base_display_name = match (vendor, model) {
                    (Some(v), Some(m)) => format!("{} {} ({})", v, m, device_name),
                    (None, Some(m)) => format!("{} ({})", m, device_name),
                    _ => {
                        // If no model, try to get a better name from the mounted partition
                        if let Some((dev_name, _)) =
                            mounted.iter().find(|(k, _)| k.starts_with(&device_name))
                        {
                            // Extract label from mount point if it looks like a label
                            if let Some((mount_point, _)) = mounted.get(dev_name) {
                                if let Some(label) = mount_point.rsplit('/').next() {
                                    if !label.is_empty()
                                        && !label.starts_with("sd")
                                        && label.len() > 2
                                    {
                                        format!("{} ({})", label, device_name)
                                    } else {
                                        device_name.clone()
                                    }
                                } else {
                                    device_name.clone()
                                }
                            } else {
                                device_name.clone()
                            }
                        } else {
                            device_name.clone()
                        }
                    }
                };

                // Check if this device or its partitions are mounted
                let (mount_info, display_name) =
                    if let Some((mount_point, _)) = mounted.get(&device_name) {
                        (mount_point.clone(), base_display_name)
                    } else {
                        // Check for mounted partitions
                        let mut partition_mount = None;
                        for (mounted_dev, (mount_point, _)) in &mounted {
                            if mounted_dev.starts_with(&device_name) {
                                partition_mount = Some(mount_point.clone());
                                break;
                            }
                        }

                        if let Some(mount_point) = partition_mount {
                            (mount_point, base_display_name)
                        } else {
                            (format!("/dev/{}", device_name), base_display_name)
                        }
                    };

                let device_path = format!("/dev/{}", device_name);

                drives.push(DriveInfo::with_constraints(
                    display_name,
                    mount_info,
                    size_gb,
                    device_path,
                    is_system,
                    is_read_only,
                ));
            }
        }
    }

    // Sort drives: removable first, then by name
    drives.sort_by(|a, b| a.name.cmp(&b.name));

    drives
}

/// Check if a device is a system drive
///
/// A system drive is one that contains critical system paths like /, /boot, /usr, etc.
fn check_if_system_drive(device_name: &str) -> bool {
    // Check if this device or any of its partitions are mounted at system locations
    let system_mount_points = [
        "/", "/boot", "/usr", "/var", "/home", "/opt", "/etc", "/bin", "/sbin", "/lib", "/lib64",
    ];

    // Get mounted filesystems
    let disks = Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let disk_name = disk.name().to_string_lossy().to_string();
        let mount_point = disk.mount_point().to_string_lossy().to_string();

        // Check if this disk belongs to our device
        if disk_name.contains(device_name)
            || disk_name.starts_with(&format!("/dev/{}", device_name))
        {
            // Check if mounted at a system location
            for sys_mount in &system_mount_points {
                if mount_point == *sys_mount {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a device is read-only
///
/// Checks the 'ro' flag in /sys/block to determine if a device is read-only.
fn check_if_read_only(device_name: &str) -> bool {
    let ro_path = format!("/sys/block/{}/ro", device_name);

    if let Ok(content) = std::fs::read_to_string(&ro_path) {
        if let Ok(value) = content.trim().parse::<u8>() {
            return value == 1;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_drives_sync() {
        // We can't easily test async in unit tests without a runtime,
        // but we can test the synchronous parts
        let disks = Disks::new_with_refreshed_list();
        // Just verify it doesn't crash - any result is valid
        let _ = disks.list();
    }

    #[test]
    fn test_check_if_read_only() {
        // This test will check the actual system, so results may vary
        // Just verify it doesn't crash
        let _ = check_if_read_only("sda");
    }

    #[test]
    fn test_check_if_system_drive() {
        // This test will check the actual system, so results may vary
        // Just verify it doesn't crash
        let _ = check_if_system_drive("sda");
    }
}
