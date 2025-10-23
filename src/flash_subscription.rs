//! Flash Subscription - Real-time progress streaming
//!
//! This module implements an Iced Subscription that monitors the flash
//! operation and emits progress updates in real-time.

use futures::SinkExt;
use iced::stream;
use iced::Subscription;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Progress event from the flash operation
#[derive(Debug, Clone)]
pub enum FlashProgress {
    /// Progress percentage (0.0 to 1.0)
    Progress(f32),
    /// Status message
    Message(String),
    /// Operation completed successfully
    Completed,
    /// Operation failed with error message
    Failed(String),
}

/// Create a subscription that monitors flash progress
pub fn flash_progress(image_path: PathBuf, device_path: PathBuf) -> Subscription<FlashProgress> {
    // Create a unique ID for this subscription based on the paths
    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    device_path.hash(&mut hasher);
    let id = hasher.finish();

    Subscription::run_with_id(
        id,
        stream::channel(100, move |mut output| async move {
            // Run the flash operation and stream progress
            let result =
                run_flash_operation(image_path.clone(), device_path.clone(), &mut output).await;

            match result {
                Ok(_) => {
                    let _ = output.send(FlashProgress::Completed).await;
                }
                Err(e) => {
                    let _ = output.send(FlashProgress::Failed(e)).await;
                }
            }

            // Subscription ends after completion
            std::future::pending().await
        }),
    )
}

/// Run the flash operation with progress reporting
async fn run_flash_operation(
    image_path: PathBuf,
    device_path: PathBuf,
    output: &mut futures::channel::mpsc::Sender<FlashProgress>,
) -> Result<(), String> {
    let image_path_str = image_path
        .to_str()
        .ok_or_else(|| "Invalid image path".to_string())?;

    let device_path_str = device_path
        .to_str()
        .ok_or_else(|| "Invalid device path".to_string())?;

    // Get image size for progress calculation
    let image_size = image_path
        .metadata()
        .map_err(|e| format!("Cannot read image file: {}", e))?
        .len();

    // Send initial message
    let _ = output
        .send(FlashProgress::Message(
            "Preparing flash operation...".to_string(),
        ))
        .await;

    // Create the flash script
    let script_content = format!(
        r#"#!/bin/bash
set -e

IMAGE="{}"
DEVICE="{}"

echo "STATUS:Starting flash operation"

# Unmount all partitions
for part in "${{DEVICE}}"*[0-9] "${{DEVICE}}p"*[0-9]; do
    if [ -e "$part" ]; then
        if mountpoint -q "$part" 2>/dev/null || grep -qs "$part" /proc/mounts; then
            echo "STATUS:Unmounting $part"
            umount "$part" 2>/dev/null || true
        fi
    fi
done

# Get image size
IMAGE_SIZE=$(stat -c%s "$IMAGE")
echo "STATUS:Image size: $IMAGE_SIZE bytes"

# Write the image using dd with progress
echo "STATUS:Writing image to device..."
dd if="$IMAGE" of="$DEVICE" bs=4M status=progress oflag=direct conv=fsync 2>&1

# Sync
echo "STATUS:Syncing data to disk..."
sync

echo "STATUS:Flash operation completed!"
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

    // Send message about privilege request
    let _ = output
        .send(FlashProgress::Message(
            "Requesting elevated privileges...".to_string(),
        ))
        .await;

    // Execute with pkexec
    let mut child = Command::new("pkexec")
        .arg("bash")
        .arg(script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Spawn a thread to read stderr (dd progress)
    let image_size_clone = image_size;
    let mut output_clone = output.clone();
    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                // Parse dd progress output
                // Format: "123456789 bytes (123 MB) copied, 5 s, 24.7 MB/s"
                if line.contains("bytes") && (line.contains("copied") || line.contains("MB/s")) {
                    if let Some(bytes_str) = line.split_whitespace().next() {
                        if let Ok(bytes_written) = bytes_str.parse::<u64>() {
                            let progress =
                                (bytes_written as f64 / image_size_clone as f64).min(1.0);
                            let _ = futures::executor::block_on(
                                output_clone.send(FlashProgress::Progress(progress as f32)),
                            );
                        }
                    }
                }
            }
        }
    });

    // Read stdout for status messages
    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("STATUS:") {
                let message = line.strip_prefix("STATUS:").unwrap_or(&line).to_string();
                let _ = output.send(FlashProgress::Message(message)).await;
            }
        }
    }

    // Wait for stderr thread
    let _ = stderr_handle.join();

    // Wait for process completion
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

    Ok(())
}
