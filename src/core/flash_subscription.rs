//! Flash Subscription - Real-time progress streaming
//!
//! This module implements an Iced Subscription that monitors the flash
//! operation and emits progress updates in real-time.

use crate::flash_debug;
use futures::SinkExt;
use iced::stream;
use iced::Subscription;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Progress event from the flash operation
#[derive(Debug, Clone)]
pub enum FlashProgress {
    /// Progress with details (percentage, bytes_written, speed_mb_per_sec)
    Progress(f32, u64, f32),
    /// Status message
    Message(String),
    /// Operation completed successfully
    Completed,
    /// Operation failed with error message
    Failed(String),
}

/// Create a subscription that monitors flash progress
pub fn flash_progress(
    image_path: PathBuf,
    device_path: PathBuf,
    cancel_token: Arc<AtomicBool>,
) -> Subscription<FlashProgress> {
    // Create a unique ID for this subscription based on the paths
    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    device_path.hash(&mut hasher);
    let id = hasher.finish();

    Subscription::run_with_id(
        id,
        stream::channel(100, move |mut output| async move {
            // Run the flash operation and stream progress
            let result = run_flash_operation(
                image_path.clone(),
                device_path.clone(),
                output.clone(),
                cancel_token.clone(),
            )
            .await;

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
    mut output: futures::channel::mpsc::Sender<FlashProgress>,
    cancel_token: Arc<AtomicBool>,
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
dd if="$IMAGE" of="$DEVICE" bs=4M status=progress oflag=direct conv=fsync

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

    // Use async_std or tokio to handle the streams
    // Since we're in an async context, we need to spawn tasks properly
    let (progress_tx, mut progress_rx) = futures::channel::mpsc::channel::<(f32, u64, f32)>(100);
    let (status_tx, mut status_rx) = futures::channel::mpsc::channel::<String>(100);

    // Spawn stdout reader in a separate thread (for both status messages and dd progress)
    let image_size_clone = image_size;
    let progress_tx_clone = progress_tx.clone();
    let cancel_token_clone1 = cancel_token.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut stdout_reader = BufReader::new(stdout);
        let mut buffer = vec![0u8; 8192];
        let mut accumulated = String::new();

        loop {
            // Check for cancellation to stop processing early
            if cancel_token_clone1.load(Ordering::SeqCst) {
                flash_debug!("stdout reader thread: cancellation detected, exiting");
                break;
            }

            match stdout_reader.read(&mut buffer) {
                Ok(0) => {
                    flash_debug!("stdout EOF reached");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    accumulated.push_str(&chunk);
                    flash_debug!(
                        "stdout chunk ({} bytes): {:?}",
                        n,
                        &chunk[..chunk.len().min(100)]
                    );

                    // Process STATUS: messages
                    for line in accumulated.lines() {
                        if line.starts_with("STATUS:") {
                            let message = line.strip_prefix("STATUS:").unwrap_or(line).to_string();
                            let _ = status_tx.clone().try_send(message);
                        }
                    }

                    // Process dd progress updates (separated by \r)
                    let parts: Vec<&str> = accumulated.split('\r').collect();
                    if parts.len() > 1 {
                        for part in &parts[..parts.len() - 1] {
                            if !part.is_empty()
                                && part.contains("bytes")
                                && !part.starts_with("STATUS:")
                            {
                                flash_debug!("Found progress: {}", part);
                                // Parse dd output: "123456789 bytes (123 MB, 117 MiB) copied..."
                                let parts_vec: Vec<&str> = part.split_whitespace().collect();

                                if let Some(bytes_str) = parts_vec.first() {
                                    if let Ok(bytes_written) = bytes_str.parse::<u64>() {
                                        let progress = (bytes_written as f64
                                            / image_size_clone as f64)
                                            .min(1.0);

                                        // Parse speed (last token before the last, e.g., "4.0" from "4.0 MB/s")
                                        let speed_mb_s = if parts_vec.len() >= 2 {
                                            parts_vec[parts_vec.len() - 2]
                                                .replace(',', ".")
                                                .parse::<f32>()
                                                .unwrap_or(0.0)
                                        } else {
                                            0.0
                                        };

                                        flash_debug!(
                                            "Progress: {:.1}% ({} / {} bytes) @ {:.1} MB/s",
                                            progress * 100.0,
                                            bytes_written,
                                            image_size_clone,
                                            speed_mb_s
                                        );
                                        let _ = progress_tx_clone.clone().try_send((
                                            progress as f32,
                                            bytes_written,
                                            speed_mb_s,
                                        ));
                                    }
                                }
                            }
                        }
                        accumulated = parts[parts.len() - 1].to_string();
                    }
                }
                Err(_e) => {
                    flash_debug!("stdout read error: {}", _e);
                    break;
                }
            }
        }
        flash_debug!("stdout reader thread exiting");
    });

    // Spawn stderr reader in a separate thread (for dd progress)
    let cancel_token_clone2 = cancel_token.clone();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut stderr_reader = BufReader::new(stderr);
        let mut buffer = vec![0u8; 8192];
        let mut accumulated = String::new();

        loop {
            // Check for cancellation to stop processing early
            if cancel_token_clone2.load(Ordering::SeqCst) {
                flash_debug!("stderr reader thread: cancellation detected, exiting");
                break;
            }

            match stderr_reader.read(&mut buffer) {
                Ok(0) => {
                    flash_debug!("stderr EOF reached");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    accumulated.push_str(&chunk);
                    flash_debug!(
                        "stderr chunk ({} bytes): {:?}",
                        n,
                        &chunk[..chunk.len().min(200)]
                    );

                    // Process dd progress updates (separated by \r - carriage return)
                    let parts: Vec<&str> = accumulated.split('\r').collect();
                    if parts.len() > 1 {
                        for part in &parts[..parts.len() - 1] {
                            if !part.is_empty() && part.contains("bytes") {
                                flash_debug!("Found dd progress: {}", part);
                                // Parse dd output: "123456789 bytes (123 MB, 117 MiB) copied..."
                                let parts_vec: Vec<&str> = part.split_whitespace().collect();

                                if let Some(bytes_str) = parts_vec.first() {
                                    if let Ok(bytes_written) = bytes_str.parse::<u64>() {
                                        let progress = (bytes_written as f64
                                            / image_size_clone as f64)
                                            .min(1.0);

                                        // Parse speed (last token before the last, e.g., "4.0" from "4.0 MB/s")
                                        let speed_mb_s = if parts_vec.len() >= 2 {
                                            parts_vec[parts_vec.len() - 2]
                                                .replace(',', ".")
                                                .parse::<f32>()
                                                .unwrap_or(0.0)
                                        } else {
                                            0.0
                                        };

                                        flash_debug!(
                                            "Progress: {:.1}% ({} / {} bytes) @ {:.1} MB/s",
                                            progress * 100.0,
                                            bytes_written,
                                            image_size_clone,
                                            speed_mb_s
                                        );
                                        let _ = progress_tx.clone().try_send((
                                            progress as f32,
                                            bytes_written,
                                            speed_mb_s,
                                        ));
                                    }
                                }
                            }
                        }
                        accumulated = parts[parts.len() - 1].to_string();
                    }
                }
                Err(_e) => {
                    flash_debug!("stderr read error: {}", _e);
                    break;
                }
            }
        }
        flash_debug!("stderr reader thread exiting");
    });

    // Forward progress and status updates to the output channel
    let mut last_progress = 0.0_f32;
    loop {
        // Check for cancellation request FIRST before processing any updates
        if cancel_token.load(Ordering::SeqCst) {
            flash_debug!("Cancellation requested, killing child process");
            // Kill the child process
            let _ = child.kill();
            // Clean up script
            let _ = std::fs::remove_file(script_path);
            return Err("Flash operation cancelled by user".to_string());
        }

        // Check if child process has exited first
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                // Process still running, check for updates
            }
            Err(e) => {
                return Err(format!("Error checking process status: {}", e));
            }
        }

        // Try to get progress update (non-blocking)
        if let Ok(Some((p, bytes, speed))) = progress_rx.try_next() {
            // Check cancellation again before sending to UI
            if cancel_token.load(Ordering::SeqCst) {
                flash_debug!("Cancellation detected, discarding progress update");
                continue;
            }

            flash_debug!(
                "Received progress from channel: {:.1}% @ {:.1} MB/s",
                p * 100.0,
                speed
            );
            // Only send if progress changed significantly (avoid flooding)
            if (p - last_progress).abs() > 0.01 || p >= 1.0 {
                flash_debug!("Sending progress to UI: {:.1}%", p * 100.0);
                let _ = output.send(FlashProgress::Progress(p, bytes, speed)).await;
                last_progress = p;
            } else {
                flash_debug!(
                    "Skipping progress update (too small change): {:.1}% (last: {:.1}%)",
                    p * 100.0,
                    last_progress * 100.0
                );
            }
        }

        // Try to get status update (non-blocking)
        if let Ok(Some(msg)) = status_rx.try_next() {
            // Check cancellation again before sending to UI
            if cancel_token.load(Ordering::SeqCst) {
                flash_debug!("Cancellation detected, discarding status update");
                continue;
            }
            let _ = output.send(FlashProgress::Message(msg)).await;
        }

        // Small delay to avoid busy-waiting
        futures_timer::Delay::new(std::time::Duration::from_millis(100)).await;
    }

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
