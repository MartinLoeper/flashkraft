//! TUI Flash Runner
//!
//! Provides a Tokio-task-based flash operation that mirrors the logic in
//! [`crate::core::flash_subscription`] but uses [`tokio::sync::mpsc`] channels
//! instead of Iced subscriptions, making it suitable for the Ratatui TUI.
//!
//! ## Wire protocol (stdout of the helper process)
//!
//! | Prefix | Meaning |
//! |--------|---------|
//! | `STAGE:<name>` | Pipeline stage transition |
//! | `SIZE:<bytes>` | Total image size in bytes |
//! | `PROGRESS:<bytes>:<speed_mb_s>` | Write progress update |
//! | `LOG:<message>` | Informational status message |
//! | `ERROR:<message>` | Terminal error |

use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::mpsc;

use crate::core::flash_writer::{parse_script_line, FlashStage, ScriptLine};
use crate::tui::app::FlashEvent;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Spawn a flash operation and forward structured [`FlashEvent`]s over `tx`.
///
/// This function is `async` so it can be handed directly to `tokio::spawn`.
/// It drives the child process from a separate OS thread (blocking I/O) and
/// communicates with the async task through a Tokio channel.
pub async fn run_flash(
    image_path: PathBuf,
    device_path: PathBuf,
    cancel_token: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<FlashEvent>,
) {
    match run_flash_inner(image_path, device_path, cancel_token, tx.clone()).await {
        Ok(()) => {
            // Completed event is sent inside run_flash_inner on STAGE:DONE.
        }
        Err(e) => {
            let _ = tx.send(FlashEvent::Failed(e));
        }
    }
}

// ---------------------------------------------------------------------------
// Inner implementation
// ---------------------------------------------------------------------------

async fn run_flash_inner(
    image_path: PathBuf,
    device_path: PathBuf,
    cancel_token: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<FlashEvent>,
) -> Result<(), String> {
    // ── Validate inputs ──────────────────────────────────────────────────────
    let image_size = image_path
        .metadata()
        .map_err(|e| format!("Cannot read image file: {e}"))?
        .len();

    if image_size == 0 {
        return Err("Image file is empty.".to_string());
    }

    // ── Locate current executable ────────────────────────────────────────────
    // We re-exec ourselves with `--flash-helper` under elevated privileges,
    // exactly as the GUI version does.
    let self_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot determine current executable path: {e}"))?;

    // ── Notify UI ────────────────────────────────────────────────────────────
    let _ = tx.send(FlashEvent::Stage(
        "Requesting elevated privileges…".to_string(),
    ));

    // ── Spawn pkexec ─────────────────────────────────────────────────────────
    let image_str = image_path
        .to_str()
        .ok_or("Image path contains invalid UTF-8")?;
    let device_str = device_path
        .to_str()
        .ok_or("Device path contains invalid UTF-8")?;

    let mut child = Command::new("pkexec")
        .arg(&self_exe)
        .arg("--flash-helper")
        .arg(image_str)
        .arg(device_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn pkexec: {e}"))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture child stdout")?;

    // ── Reader thread ────────────────────────────────────────────────────────
    // Blocking reads happen in an OS thread; lines are forwarded to the async
    // Tokio task via a std channel (converted to Tokio on the other end).
    let (line_tx, mut line_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let cancel_reader = cancel_token.clone();

    std::thread::spawn(move || {
        let mut buf = vec![0u8; 65_536];
        let mut accumulated = String::new();

        loop {
            if cancel_reader.load(Ordering::SeqCst) {
                break;
            }

            match stdout.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    accumulated.push_str(&String::from_utf8_lossy(&buf[..n]));

                    // Split on \r and \n (handles both dd-style \r progress
                    // and normal newline-delimited helper output).
                    loop {
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
                                let line = accumulated[..pos].trim().to_string();
                                accumulated = accumulated[pos + 1..].to_string();
                                if !line.is_empty() && line_tx.send(line).is_err() {
                                    return; // receiver dropped (cancelled)
                                }
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        // Flush remainder.
        let remainder = accumulated.trim().to_string();
        if !remainder.is_empty() {
            let _ = line_tx.send(remainder);
        }
    });

    // ── Async event loop ─────────────────────────────────────────────────────
    let mut total_size: u64 = image_size;
    let mut last_progress: f32 = 0.0;

    loop {
        // ── Cancellation ────────────────────────────────────────────────────
        if cancel_token.load(Ordering::SeqCst) {
            let _ = child.kill();
            return Err("Flash operation cancelled by user.".to_string());
        }

        // ── Check child exit ─────────────────────────────────────────────────
        match child.try_wait() {
            Ok(Some(status)) => {
                // Drain any remaining lines before deciding outcome.
                drain_lines(&mut line_rx, &tx, &mut total_size, &mut last_progress).await;

                return if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "Flash helper exited with code {:?}",
                        status.code().unwrap_or(-1)
                    ))
                };
            }
            Ok(None) => {} // still running
            Err(e) => return Err(format!("Error polling child process: {e}")),
        }

        // ── Process available lines (non-blocking) ───────────────────────────
        while let Ok(line) = line_rx.try_recv() {
            if cancel_token.load(Ordering::SeqCst) {
                break;
            }
            process_line(&line, &tx, &mut total_size, &mut last_progress, &mut child)?;
        }

        // Yield for ~100 ms to avoid a busy-wait loop.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

// ---------------------------------------------------------------------------
// Line processing
// ---------------------------------------------------------------------------

fn process_line(
    line: &str,
    tx: &mpsc::UnboundedSender<FlashEvent>,
    total_size: &mut u64,
    last_progress: &mut f32,
    child: &mut std::process::Child,
) -> Result<(), String> {
    match parse_script_line(line) {
        ScriptLine::Stage(stage) => {
            let label = stage.to_string();
            let _ = tx.send(FlashEvent::Stage(label));

            if stage == FlashStage::Done {
                let _ = child.wait();
                let _ = tx.send(FlashEvent::Completed);
                // Signal outer loop to exit cleanly.
                return Err("__done__".to_string());
            }
        }

        ScriptLine::Size(n) => {
            if n > 0 {
                *total_size = n;
            }
        }

        ScriptLine::Progress(bytes, speed) => {
            if *total_size > 0 {
                let p = (bytes as f64 / *total_size as f64).clamp(0.0, 1.0) as f32;
                if (p - *last_progress).abs() >= 0.005 || p >= 1.0 {
                    let _ = tx.send(FlashEvent::Progress(p, bytes, speed));
                    *last_progress = p;
                }
            }
        }

        ScriptLine::DdProgress(bytes, speed) => {
            if *total_size > 0 {
                let p = (bytes as f64 / *total_size as f64).clamp(0.0, 1.0) as f32;
                if (p - *last_progress).abs() >= 0.005 || p >= 1.0 {
                    let _ = tx.send(FlashEvent::Progress(p, bytes, speed));
                    *last_progress = p;
                }
            }
        }

        ScriptLine::DdExit(0) => {
            if *last_progress < 1.0 {
                let _ = tx.send(FlashEvent::Progress(1.0, *total_size, 0.0));
                *last_progress = 1.0;
            }
        }

        ScriptLine::DdExit(code) => {
            let _ = tx.send(FlashEvent::Log(format!("dd exited with code {code}")));
        }

        ScriptLine::Error(msg) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(msg);
        }

        ScriptLine::Log(msg) => {
            let _ = tx.send(FlashEvent::Log(msg));
        }

        ScriptLine::Unknown(_) => {}
    }

    Ok(())
}

/// Drain remaining lines from the reader after the child exits.
async fn drain_lines(
    rx: &mut mpsc::UnboundedReceiver<String>,
    tx: &mpsc::UnboundedSender<FlashEvent>,
    total_size: &mut u64,
    last_progress: &mut f32,
) {
    // Give the reader thread a tiny moment to flush its last lines.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    while let Ok(line) = rx.try_recv() {
        match parse_script_line(&line) {
            ScriptLine::Stage(FlashStage::Done) => {
                let _ = tx.send(FlashEvent::Completed);
            }
            ScriptLine::Stage(stage) => {
                let _ = tx.send(FlashEvent::Stage(stage.to_string()));
            }
            ScriptLine::Error(msg) => {
                let _ = tx.send(FlashEvent::Failed(msg));
            }
            ScriptLine::Log(msg) => {
                let _ = tx.send(FlashEvent::Log(msg));
            }
            ScriptLine::Progress(bytes, speed) => {
                if *total_size > 0 {
                    let p = (bytes as f64 / *total_size as f64).clamp(0.0, 1.0) as f32;
                    if (p - *last_progress).abs() >= 0.005 || p >= 1.0 {
                        let _ = tx.send(FlashEvent::Progress(p, bytes, speed));
                        *last_progress = p;
                    }
                }
            }
            ScriptLine::DdProgress(bytes, speed) => {
                if *total_size > 0 {
                    let p = (bytes as f64 / *total_size as f64).clamp(0.0, 1.0) as f32;
                    if (p - *last_progress).abs() >= 0.005 || p >= 1.0 {
                        let _ = tx.send(FlashEvent::Progress(p, bytes, speed));
                        *last_progress = p;
                    }
                }
            }
            ScriptLine::DdExit(0) => {
                if *last_progress < 1.0 {
                    let _ = tx.send(FlashEvent::Progress(1.0, *total_size, 0.0));
                    *last_progress = 1.0;
                }
            }
            ScriptLine::Size(n) => {
                if n > 0 {
                    *total_size = n;
                }
            }
            _ => {}
        }
    }
}
