//! Flash Subscription - Real-time progress streaming
//!
//! This module implements an Iced [`Subscription`] that drives the flash
//! pipeline and forwards structured progress events to the UI.
//!
//! ## Architecture
//!
//! The flash operation is performed entirely in pure Rust by the **same
//! binary** re-executed with elevated privileges via `pkexec`:
//!
//! ```text
//! pkexec /path/to/flashkraft --flash-helper <image_path> <device_path>
//! ```
//!
//! `main.rs` detects `--flash-helper` before touching any GUI code and
//! dispatches to [`flashkraft_core::flash_helper::run`].  The helper writes all
//! output — progress, stage transitions, logs, and errors — to **stdout**.
//! This subscription spawns a single blocking reader thread that ships those
//! lines to an async channel, where the main loop parses them and forwards
//! the appropriate [`FlashProgress`] events to the Iced runtime.
//!
//! ## Wire protocol (stdout of the helper process)
//!
//! | Prefix | Parsed as | Meaning |
//! |--------|-----------|---------|
//! | `STAGE:<name>` | [`ScriptLine::Stage`] | Pipeline stage transition |
//! | `SIZE:<bytes>` | [`ScriptLine::Size`] | Total image size in bytes |
//! | `PROGRESS:<bytes>:<speed_mb_s>` | [`ScriptLine::Progress`] | Write progress update |
//! | `LOG:<message>` | [`ScriptLine::Log`] | Informational status message |
//! | `ERROR:<message>` | [`ScriptLine::Error`] | Terminal error (process exits non-zero) |
//!
//! All parsing is handled by [`crate::core::flash_writer::parse_script_line`].
//!
//! ## Cancellation
//!
//! An [`AtomicBool`] cancel token is shared between the Iced update loop and
//! this subscription.  Setting the token causes the subscription to kill the
//! child process and return an error on the next poll cycle (~100 ms latency).
//!
//! ## Reader thread design
//!
//! Raw bytes from the helper's stdout are accumulated in a `String` buffer.
//! The buffer is split on **both `\r` and `\n`** so that `dd status=progress`
//! carriage-return updates (legacy compat) are handled identically to normal
//! newline-delimited lines from the Rust helper.

use crate::core::flash_writer::{parse_script_line, FlashStage, ScriptLine};
use crate::flash_debug;
use futures::SinkExt;
use iced::stream;
use iced::Subscription;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Progress event emitted by the flash subscription.
#[derive(Debug, Clone)]
pub enum FlashProgress {
    /// `(progress 0.0–1.0, bytes_written, speed_mb_s)`
    Progress(f32, u64, f32),
    /// Human-readable status message for the UI
    Message(String),
    /// The flash operation finished successfully
    Completed,
    /// The flash operation failed with an error message
    Failed(String),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a subscription that streams [`FlashProgress`] events while the
/// flash operation runs.
pub fn flash_progress(
    image_path: PathBuf,
    device_path: PathBuf,
    cancel_token: Arc<AtomicBool>,
) -> Subscription<FlashProgress> {
    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    device_path.hash(&mut hasher);
    let id = hasher.finish();

    Subscription::run_with_id(
        id,
        stream::channel(100, move |mut output| async move {
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

            // Keep the subscription alive (Iced requirement).
            std::future::pending().await
        }),
    )
}

// ---------------------------------------------------------------------------
// Internal implementation
// ---------------------------------------------------------------------------

/// Run the flash operation and forward progress events to `output`.
///
/// Returns `Ok(())` when the script emits `STAGE:DONE`, or `Err(message)`
/// on any failure (cancellation, script error, verification failure, …).
async fn run_flash_operation(
    image_path: PathBuf,
    device_path: PathBuf,
    mut output: futures::channel::mpsc::Sender<FlashProgress>,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), String> {
    let image_path_str = image_path
        .to_str()
        .ok_or_else(|| "Invalid image path (non-UTF-8)".to_string())?;

    let device_path_str = device_path
        .to_str()
        .ok_or_else(|| "Invalid device path (non-UTF-8)".to_string())?;

    // Verify the image file is readable before we even ask for privileges.
    let image_size = image_path
        .metadata()
        .map_err(|e| format!("Cannot read image file: {e}"))?
        .len();

    if image_size == 0 {
        return Err("Image file is empty".to_string());
    }

    // ── Locate the current executable ────────────────────────────────────────
    // We re-launch ourselves with elevated privileges in `--flash-helper` mode.
    // Using the running binary's path means no separate helper binary needs to
    // be installed — distribution is a single file.
    let self_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot determine current executable path: {e}"))?;

    // ── Notify the UI ────────────────────────────────────────────────────────
    let _ = output
        .send(FlashProgress::Message(
            "Requesting elevated privileges…".to_string(),
        ))
        .await;

    // ── Spawn pkexec ─────────────────────────────────────────────────────────
    // We re-execute this binary as root with the `--flash-helper` flag.
    // The helper writes all output (including errors) to stdout, so we only
    // need to capture stdout here.
    let mut child = Command::new("pkexec")
        .arg(&self_exe)
        .arg("--flash-helper")
        .arg(image_path_str)
        .arg(device_path_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // helper writes everything to stdout
        .spawn()
        .map_err(|e| format!("Failed to spawn pkexec: {e}"))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture child stdout")?;

    // ── Spawn a reader thread ─────────────────────────────────────────────────
    // We read raw bytes off the child's stdout in a blocking thread and ship
    // them to an async channel.  This decouples the blocking I/O from the
    // async executor.
    let (line_tx, mut line_rx) = futures::channel::mpsc::channel::<String>(512);
    let cancel_clone = cancel_token.clone();

    std::thread::spawn(move || {
        let mut buf = vec![0u8; 65536];
        let mut accumulated = String::new();

        loop {
            if cancel_clone.load(Ordering::SeqCst) {
                flash_debug!("reader thread: cancellation detected, stopping");
                break;
            }

            match stdout.read(&mut buf) {
                Ok(0) => {
                    flash_debug!("reader thread: EOF");
                    break;
                }
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    accumulated.push_str(&chunk);

                    // Split on both \r and \n so we catch dd's in-place
                    // carriage-return updates as well as regular newlines.
                    loop {
                        // Find the earliest \r or \n
                        let cr = accumulated.find('\r');
                        let nl = accumulated.find('\n');

                        let split_at = match (cr, nl) {
                            (Some(a), Some(b)) => Some(a.min(b)),
                            (Some(a), None) => Some(a),
                            (None, Some(b)) => Some(b),
                            (None, None) => None,
                        };

                        match split_at {
                            None => break,
                            Some(pos) => {
                                let line = accumulated[..pos].to_string();
                                // Advance past the separator character
                                accumulated = accumulated[pos + 1..].to_string();

                                let trimmed = line.trim().to_string();
                                if !trimmed.is_empty() && line_tx.clone().try_send(trimmed).is_err()
                                {
                                    // Receiver dropped — cancellation likely.
                                    return;
                                }
                            }
                        }
                    }
                }
                Err(_e) => {
                    flash_debug!("reader thread: read error: {_e}");
                    break;
                }
            }
        }

        // Flush any remainder that didn't end with a separator.
        let remainder = accumulated.trim().to_string();
        if !remainder.is_empty() {
            let _ = line_tx.clone().try_send(remainder);
        }

        flash_debug!("reader thread: exiting");
    });

    // ── Main async event loop ─────────────────────────────────────────────────
    let mut last_progress: f32 = 0.0;
    // `image_size` is known upfront from the file metadata; the script also
    // emits `SIZE:<n>` which we use as a cross-check.
    let mut total_size: u64 = image_size;

    loop {
        // Check cancellation first.
        if cancel_token.load(Ordering::SeqCst) {
            flash_debug!("cancellation requested — killing child");
            let _ = child.kill();
            return Err("Flash operation cancelled by user".to_string());
        }

        // Check if child has already exited.
        match child.try_wait() {
            Ok(Some(status)) => {
                flash_debug!("child exited: {status:?}");
                // Drain any remaining lines from the reader thread before
                // evaluating the outcome.
                drain_remaining(&mut line_rx, &mut output, &mut last_progress, total_size).await;

                if status.success() {
                    return Ok(());
                } else {
                    return Err(format!(
                        "Flash helper exited with code {:?}",
                        status.code().unwrap_or(-1)
                    ));
                }
            }
            Ok(None) => {
                // Still running — process pending lines below.
            }
            Err(e) => {
                return Err(format!("Error polling child process: {e}"));
            }
        }

        // Process all lines currently available in the channel (non-blocking).
        while let Ok(line) = line_rx.try_recv() {
            if cancel_token.load(Ordering::SeqCst) {
                break;
            }

            flash_debug!("line: {line:?}");

            match parse_script_line(&line) {
                // ── Stage transitions ────────────────────────────────────────
                ScriptLine::Stage(stage) => {
                    let msg = stage.to_string();
                    flash_debug!("stage → {msg}");
                    let _ = output.send(FlashProgress::Message(msg)).await;

                    if stage == FlashStage::Done {
                        // Helper finished successfully.
                        let _ = child.wait(); // reap
                        return Ok(());
                    }
                }

                // ── Total size (for progress ratio) ──────────────────────────
                ScriptLine::Size(n) => {
                    flash_debug!("total size from helper: {n}");
                    if n > 0 {
                        total_size = n;
                    }
                }

                // ── Pure-Rust helper progress line ───────────────────────────
                // Format: PROGRESS:<bytes_written>:<speed_mb_s>
                ScriptLine::Progress(bytes_written, speed_mb_s) => {
                    if total_size == 0 {
                        continue;
                    }
                    let progress =
                        (bytes_written as f64 / total_size as f64).clamp(0.0, 1.0) as f32;

                    flash_debug!(
                        "progress: {:.1}% ({bytes_written}/{total_size}) @ {speed_mb_s:.1} MB/s",
                        progress * 100.0
                    );

                    // Throttle: only forward if progress moved by ≥ 0.5 %
                    if (progress - last_progress).abs() >= 0.005 || progress >= 1.0 {
                        let _ = output
                            .send(FlashProgress::Progress(progress, bytes_written, speed_mb_s))
                            .await;
                        last_progress = progress;
                    }
                }

                // ── Legacy dd progress line (kept for compat) ────────────────
                ScriptLine::DdProgress(bytes_written, speed_mb_s) => {
                    if total_size == 0 {
                        continue;
                    }
                    let progress =
                        (bytes_written as f64 / total_size as f64).clamp(0.0, 1.0) as f32;

                    if (progress - last_progress).abs() >= 0.005 || progress >= 1.0 {
                        let _ = output
                            .send(FlashProgress::Progress(progress, bytes_written, speed_mb_s))
                            .await;
                        last_progress = progress;
                    }
                }

                // ── Legacy dd exit code ──────────────────────────────────────
                ScriptLine::DdExit(0) => {
                    flash_debug!("dd exited successfully (legacy)");
                    if last_progress < 1.0 {
                        let _ = output
                            .send(FlashProgress::Progress(1.0, total_size, 0.0))
                            .await;
                        last_progress = 1.0;
                    }
                }
                ScriptLine::DdExit(code) => {
                    flash_debug!("dd exited with non-zero code {code} (legacy)");
                    let _ = output
                        .send(FlashProgress::Message(format!(
                            "dd exited with code {code}"
                        )))
                        .await;
                }

                // ── Helper-level error ───────────────────────────────────────
                ScriptLine::Error(msg) => {
                    flash_debug!("helper error: {msg}");
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(msg);
                }

                // ── Informational log ────────────────────────────────────────
                ScriptLine::Log(msg) => {
                    flash_debug!("log: {msg}");
                    let _ = output.send(FlashProgress::Message(msg)).await;
                }

                // ── Unknown / unrecognised output ────────────────────────────
                ScriptLine::Unknown(_raw) => {
                    flash_debug!("unknown line: {_raw:?}");
                    // Silently ignore noise from pkexec / the system.
                }
            }
        }

        // Yield for ~100 ms to avoid a busy-wait loop.
        futures_timer::Delay::new(std::time::Duration::from_millis(100)).await;
    }
}

/// Drain any remaining lines from the reader after the child exits.
async fn drain_remaining(
    line_rx: &mut futures::channel::mpsc::Receiver<String>,
    output: &mut futures::channel::mpsc::Sender<FlashProgress>,
    last_progress: &mut f32,
    total_size: u64,
) {
    while let Ok(line) = line_rx.try_recv() {
        flash_debug!("drain: {line:?}");
        match parse_script_line(&line) {
            ScriptLine::Stage(FlashStage::Done) => {
                let _ = output
                    .send(FlashProgress::Message("Flash complete!".to_string()))
                    .await;
            }
            ScriptLine::Stage(stage) => {
                let _ = output.send(FlashProgress::Message(stage.to_string())).await;
            }
            ScriptLine::Error(msg) => {
                let _ = output.send(FlashProgress::Failed(msg)).await;
            }
            ScriptLine::Log(msg) => {
                let _ = output.send(FlashProgress::Message(msg)).await;
            }
            // Pure-Rust helper structured progress
            ScriptLine::Progress(bytes_written, speed_mb_s) => {
                if total_size > 0 {
                    let progress =
                        (bytes_written as f64 / total_size as f64).clamp(0.0, 1.0) as f32;
                    if (progress - *last_progress).abs() >= 0.005 || progress >= 1.0 {
                        let _ = output
                            .send(FlashProgress::Progress(progress, bytes_written, speed_mb_s))
                            .await;
                        *last_progress = progress;
                    }
                }
            }
            // Legacy dd progress
            ScriptLine::DdProgress(bytes_written, speed_mb_s) => {
                if total_size > 0 {
                    let progress =
                        (bytes_written as f64 / total_size as f64).clamp(0.0, 1.0) as f32;
                    if (progress - *last_progress).abs() >= 0.005 || progress >= 1.0 {
                        let _ = output
                            .send(FlashProgress::Progress(progress, bytes_written, speed_mb_s))
                            .await;
                        *last_progress = progress;
                    }
                }
            }
            ScriptLine::DdExit(0) => {
                if *last_progress < 1.0 {
                    let _ = output
                        .send(FlashProgress::Progress(1.0, total_size, 0.0))
                        .await;
                    *last_progress = 1.0;
                }
            }
            ScriptLine::Size(_n) => {
                flash_debug!("drain: size = {_n}");
            }
            ScriptLine::DdExit(_code) => {
                flash_debug!("drain: dd exit code {_code}");
            }
            ScriptLine::Unknown(_) => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Verify that `FlashProgress` variants are `Clone` (required by Iced).
    #[test]
    fn test_flash_progress_clone() {
        let p = FlashProgress::Progress(0.5, 512, 10.0);
        let _cloned = p.clone();

        let m = FlashProgress::Message("hello".to_string());
        let _cloned = m.clone();

        let c = FlashProgress::Completed;
        let _cloned = c.clone();

        let f = FlashProgress::Failed("oops".to_string());
        let _cloned = f.clone();
    }

    #[test]
    fn test_flash_progress_debug() {
        let p = FlashProgress::Progress(0.75, 805_306_368, 25.5);
        let s = format!("{p:?}");
        assert!(s.contains("Progress"));
        assert!(s.contains("0.75"));
    }

    /// Ensure the hash-based subscription ID is stable for the same paths.
    #[test]
    fn test_subscription_id_is_deterministic() {
        fn compute_id(img: &str, dev: &str) -> u64 {
            let mut h = DefaultHasher::new();
            PathBuf::from(img).hash(&mut h);
            PathBuf::from(dev).hash(&mut h);
            h.finish()
        }

        let id1 = compute_id("/tmp/a.iso", "/dev/sdb");
        let id2 = compute_id("/tmp/a.iso", "/dev/sdb");
        let id3 = compute_id("/tmp/b.iso", "/dev/sdb");

        assert_eq!(id1, id2, "same inputs → same ID");
        assert_ne!(id1, id3, "different image → different ID");
    }

    /// Ensure different device paths produce different subscription IDs.
    #[test]
    fn test_subscription_id_differs_for_different_devices() {
        fn compute_id(img: &str, dev: &str) -> u64 {
            let mut h = DefaultHasher::new();
            PathBuf::from(img).hash(&mut h);
            PathBuf::from(dev).hash(&mut h);
            h.finish()
        }

        let id_sdb = compute_id("/tmp/a.iso", "/dev/sdb");
        let id_sdc = compute_id("/tmp/a.iso", "/dev/sdc");
        assert_ne!(id_sdb, id_sdc);
    }
}
