//! Commands (Side Effects) - The Elm Architecture
//!
//! This module contains all async functions that perform side effects.
//! These functions are called from Commands in the update function and
//! their results are converted back into Messages.
//!
//! Commands keep the update function pure by moving all I/O operations
//! outside of the state transition logic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use rfd::AsyncFileDialog;
use sysinfo::Disks;

use crate::model::DriveInfo;

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

                drives.push(DriveInfo::new(
                    display_name,
                    mount_info,
                    size_gb,
                    device_path,
                ));
            }
        }
    }

    // Sort drives: removable first, then by name
    drives.sort_by(|a, b| a.name.cmp(&b.name));

    drives
}

/// Verify that the target device is not a system disk
///
/// # Arguments
///
/// * `device_path` - The device path to check
///
/// # Returns
///
/// `Ok(())` if safe to write, `Err(String)` if it appears to be a system disk
fn verify_not_system_disk(device_path: &str) -> Result<(), String> {
    // Check if the device is mounted at critical system paths
    let disks = Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let disk_name = disk.name().to_string_lossy();
        if disk_name.starts_with(device_path) || device_path.contains(&disk_name.to_string()) {
            let mount_point = disk.mount_point().to_string_lossy();

            // Check for system-critical mount points
            let critical_mounts = ["/", "/boot", "/usr", "/var", "/home"];
            for critical in &critical_mounts {
                if mount_point == *critical || mount_point.starts_with(&format!("{}/", critical)) {
                    return Err(format!(
                        "SAFETY CHECK FAILED: {} is mounted at {} which is a system-critical path.\n\
                         Flashing this device would destroy your system. Operation aborted.",
                        disk_name, mount_point
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Flash an image to a drive using pkexec for privilege escalation
///
/// This performs the actual disk image writing operation by creating
/// a helper script that runs with elevated privileges via pkexec.
///
/// The function will:
/// 1. Verify this is not a system disk (safety check)
/// 2. Verify the image file exists and is readable
/// 3. Create a helper script that performs the flash operation
/// 4. Use pkexec to run the script with elevated privileges
/// 5. Monitor progress and report to the user
///
/// # Arguments
///
/// * `image_path` - Path to the OS image file
/// * `device_path` - Path to the target block device (e.g., /dev/sde)
///
/// # Returns
///
/// `Ok(())` on success, `Err(String)` with error message on failure
///
/// # Safety
///
/// This function writes to raw block devices and can destroy data.
/// Ensure you've selected the correct device!
pub async fn flash_image(image_path: PathBuf, device_path: PathBuf) -> Result<(), String> {
    let device_path_str = device_path
        .to_str()
        .ok_or_else(|| "Invalid device path".to_string())?;

    let image_path_str = image_path
        .to_str()
        .ok_or_else(|| "Invalid image path".to_string())?;

    println!("========================================");
    println!("[INFO] Starting flash operation");
    println!("[INFO] Image: {}", image_path_str);
    println!("[INFO] Target device: {}", device_path_str);
    println!("========================================");

    // Safety check: Verify this is not a system disk
    println!("[1/5] Verifying device is not a system disk...");
    verify_not_system_disk(device_path_str)?;
    println!("[✓] Safety check passed");

    // Verify image file exists and is readable
    println!("[2/5] Verifying image file...");
    if !image_path.exists() {
        return Err(format!("Image file does not exist: {}", image_path_str));
    }

    let image_metadata = image_path
        .metadata()
        .map_err(|e| format!("Cannot read image file metadata: {}", e))?;

    if !image_metadata.is_file() {
        return Err(format!("Image path is not a file: {}", image_path_str));
    }

    let image_size = image_metadata.len();
    println!(
        "[✓] Image file valid: {} bytes ({:.2} GB)",
        image_size,
        image_size as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    // Verify device exists
    println!("[3/5] Verifying target device exists...");
    if !device_path.exists() {
        return Err(format!("Device does not exist: {}", device_path_str));
    }
    println!("[✓] Device exists");

    // Check if device size is sufficient
    let device_size_path = format!(
        "/sys/block/{}/size",
        device_path_str
            .strip_prefix("/dev/")
            .unwrap_or(device_path_str)
    );
    if let Ok(size_str) = std::fs::read_to_string(&device_size_path) {
        if let Ok(size_sectors) = size_str.trim().parse::<u64>() {
            let device_size = size_sectors * 512;
            if device_size < image_size {
                return Err(format!(
                    "Device is too small! Image size: {} bytes, Device size: {} bytes",
                    image_size, device_size
                ));
            }
        }
    }

    // Create a helper script for privileged operations
    println!("[4/5] Preparing flash operation...");
    let script_content = format!(
        r#"#!/bin/bash
set -e

IMAGE="{}"
DEVICE="{}"

echo "[INFO] Starting privileged flash operation"

# Unmount all partitions
for part in "${{DEVICE}}"*[0-9] "${{DEVICE}}p"*[0-9]; do
    if [ -e "$part" ]; then
        if mountpoint -q "$part" 2>/dev/null || grep -qs "$part" /proc/mounts; then
            echo "[INFO] Unmounting $part"
            umount "$part" 2>/dev/null || true
        fi
    fi
done

# Get image size
IMAGE_SIZE=$(stat -c%s "$IMAGE")
echo "[INFO] Image size: $IMAGE_SIZE bytes"

# Write the image using dd with progress
echo "[INFO] Writing image to device..."
dd if="$IMAGE" of="$DEVICE" bs=4M status=progress oflag=direct conv=fsync

# Sync
echo "[INFO] Syncing..."
sync

echo "[SUCCESS] Flash operation completed!"
"#,
        image_path_str, device_path_str
    );

    let script_path = "/tmp/flashkraft_flash.sh";
    std::fs::write(script_path, script_content)
        .map_err(|e| format!("Failed to create flash script: {}", e))?;

    // Make script executable
    Command::new("chmod")
        .arg("+x")
        .arg(script_path)
        .output()
        .map_err(|e| format!("Failed to make script executable: {}", e))?;

    println!("[✓] Flash script prepared");

    // Execute with pkexec
    println!("[5/5] Requesting elevated privileges...");
    println!("[INFO] A password prompt will appear.");
    println!("[INFO] Progress will be shown in this terminal window.");
    println!("========================================");
    println!();

    use std::io::{BufRead, BufReader};
    use std::process::Stdio;

    let mut child = Command::new("pkexec")
        .arg("bash")
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to execute pkexec: {}. Is pkexec installed?\n\
                 Alternative: Run the application with sudo",
                e
            )
        })?;

    // Read and display output in real-time
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Spawn a thread to read stderr (dd progress output)
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                // dd outputs progress to stderr
                if !line.trim().is_empty() {
                    println!("[DD] {}", line);
                }
            }
        }
    });

    // Read stdout (our script messages)
    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            println!("{}", line);
        }
    }

    // Wait for stderr thread
    let _ = stderr_handle.join();

    // Wait for process to complete
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    // Clean up script
    let _ = std::fs::remove_file(script_path);

    if !status.success() {
        return Err(format!(
            "Flash operation failed with exit code: {:?}",
            status.code()
        ));
    }

    println!();
    println!("========================================");
    println!("[SUCCESS] Flash operation completed successfully!");
    println!("[INFO] You can now safely remove the device.");
    println!("========================================");

    Ok(())
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
    fn test_verify_not_system_disk() {
        // This test just verifies the function exists and can be called
        // We can't test with real system disks in unit tests
        let result = verify_not_system_disk("/dev/null");
        // Should succeed for /dev/null
        assert!(result.is_ok());
    }
}
