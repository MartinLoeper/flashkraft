//! TUI Flash Runner
//!
//! Provides a Tokio-task-based flash operation that mirrors the logic in
//! the Iced `flash_subscription` module but uses [`tokio::sync::mpsc`] channels
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

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_channel() -> (
        mpsc::UnboundedSender<FlashEvent>,
        mpsc::UnboundedReceiver<FlashEvent>,
    ) {
        mpsc::unbounded_channel()
    }

    /// Drain all currently available events from `rx` into a `Vec`.
    fn drain(rx: &mut mpsc::UnboundedReceiver<FlashEvent>) -> Vec<FlashEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            events.push(ev);
        }
        events
    }

    // ── process_line: STAGE ───────────────────────────────────────────────────

    #[test]
    fn process_line_stage_writing_sends_stage_event() {
        let (tx, mut rx) = make_channel();
        let mut total = 1024u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        let result = process_line("STAGE:WRITING", &tx, &mut total, &mut last, &mut child);
        assert!(result.is_ok());

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], FlashEvent::Stage(s) if s.to_lowercase().contains("writ")));
        let _ = child.wait();
    }

    #[test]
    fn process_line_stage_done_sends_completed_and_returns_sentinel() {
        let (tx, mut rx) = make_channel();
        let mut total = 1024u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        let result = process_line("STAGE:DONE", &tx, &mut total, &mut last, &mut child);
        // DONE returns Err("__done__") as a clean-exit sentinel.
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "__done__");

        let events = drain(&mut rx);
        // Should have received a Stage event then Completed.
        assert!(events.iter().any(|e| matches!(e, FlashEvent::Completed)));
        let _ = child.wait();
    }

    // ── process_line: SIZE ────────────────────────────────────────────────────

    #[test]
    fn process_line_size_updates_total_size() {
        let (tx, mut rx) = make_channel();
        let mut total = 0u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        let result = process_line("SIZE:4096000000", &tx, &mut total, &mut last, &mut child);
        assert!(result.is_ok());
        assert_eq!(total, 4_096_000_000);
        assert!(drain(&mut rx).is_empty(), "SIZE emits no events");
        let _ = child.wait();
    }

    #[test]
    fn process_line_size_zero_does_not_update_total() {
        let (tx, mut rx) = make_channel();
        let mut total = 999u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line("SIZE:0", &tx, &mut total, &mut last, &mut child).ok();
        assert_eq!(total, 999, "zero-size must not overwrite existing total");
        assert!(drain(&mut rx).is_empty());
        let _ = child.wait();
    }

    // ── process_line: PROGRESS ────────────────────────────────────────────────

    #[test]
    fn process_line_progress_sends_progress_event() {
        let (tx, mut rx) = make_channel();
        let mut total = 1_000_000u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line(
            "PROGRESS:500000:10.5",
            &tx,
            &mut total,
            &mut last,
            &mut child,
        )
        .ok();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        if let FlashEvent::Progress(p, bytes, speed) = &events[0] {
            assert!((*p - 0.5).abs() < 0.01, "progress should be ~0.5");
            assert_eq!(*bytes, 500_000);
            assert!((*speed - 10.5).abs() < 0.01);
        } else {
            panic!("expected Progress event, got {:?}", events[0]);
        }
        let _ = child.wait();
    }

    #[test]
    fn process_line_progress_suppressed_when_delta_too_small() {
        let (tx, mut rx) = make_channel();
        let mut total = 1_000_000u64;
        // Set last_progress close to the new value so the delta < 0.005.
        let mut last = 0.500f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        // Sending 500001 bytes → progress ≈ 0.500001, delta ≈ 0.000001 < 0.005
        process_line(
            "PROGRESS:500001:5.0",
            &tx,
            &mut total,
            &mut last,
            &mut child,
        )
        .ok();

        assert!(
            drain(&mut rx).is_empty(),
            "tiny progress delta must be suppressed"
        );
        let _ = child.wait();
    }

    #[test]
    fn process_line_progress_zero_total_emits_nothing() {
        let (tx, mut rx) = make_channel();
        let mut total = 0u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line(
            "PROGRESS:500000:10.0",
            &tx,
            &mut total,
            &mut last,
            &mut child,
        )
        .ok();
        assert!(drain(&mut rx).is_empty(), "zero total should emit nothing");
        let _ = child.wait();
    }

    // ── process_line: LOG ─────────────────────────────────────────────────────

    #[test]
    fn process_line_log_sends_log_event() {
        let (tx, mut rx) = make_channel();
        let mut total = 1024u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line(
            "LOG:syncing buffers",
            &tx,
            &mut total,
            &mut last,
            &mut child,
        )
        .ok();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], FlashEvent::Log(msg) if msg == "syncing buffers"),
            "got {:?}",
            events[0]
        );
        let _ = child.wait();
    }

    // ── process_line: ERROR ───────────────────────────────────────────────────

    #[test]
    fn process_line_error_returns_err_with_message() {
        let (tx, mut rx) = make_channel();
        let mut total = 1024u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        let result = process_line(
            "ERROR:disk write failed",
            &tx,
            &mut total,
            &mut last,
            &mut child,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "disk write failed");
        // No events should have been sent for an error line.
        assert!(drain(&mut rx).is_empty());
        let _ = child.wait();
    }

    // ── process_line: UNKNOWN ─────────────────────────────────────────────────

    #[test]
    fn process_line_unknown_is_silently_ignored() {
        let (tx, mut rx) = make_channel();
        let mut total = 1024u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        let result = process_line("some random output", &tx, &mut total, &mut last, &mut child);
        assert!(result.is_ok());
        assert!(drain(&mut rx).is_empty());
        let _ = child.wait();
    }

    // ── process_line: DD_EXIT ─────────────────────────────────────────────────

    #[test]
    fn process_line_dd_exit_zero_sends_full_progress_when_not_complete() {
        let (tx, mut rx) = make_channel();
        let mut total = 1_000_000u64;
        let mut last = 0.5f32; // simulate partially done
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line("DD_EXIT:0", &tx, &mut total, &mut last, &mut child).ok();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], FlashEvent::Progress(p, _, _) if (*p - 1.0).abs() < 0.001),
            "DD_EXIT:0 should push progress to 1.0"
        );
        let _ = child.wait();
    }

    #[test]
    fn process_line_dd_exit_zero_skipped_when_already_complete() {
        let (tx, mut rx) = make_channel();
        let mut total = 1_000_000u64;
        let mut last = 1.0f32; // already at 100%
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line("DD_EXIT:0", &tx, &mut total, &mut last, &mut child).ok();
        assert!(
            drain(&mut rx).is_empty(),
            "should not resend full-progress when already at 1.0"
        );
        let _ = child.wait();
    }

    #[test]
    fn process_line_dd_exit_nonzero_sends_log() {
        let (tx, mut rx) = make_channel();
        let mut total = 1_000_000u64;
        let mut last = 0.0f32;
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawn true");

        process_line("DD_EXIT:1", &tx, &mut total, &mut last, &mut child).ok();

        let events = drain(&mut rx);
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], FlashEvent::Log(msg) if msg.contains("1")),
            "non-zero DD_EXIT should log the exit code"
        );
        let _ = child.wait();
    }

    // ── drain_lines ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn drain_lines_completed_on_stage_done() {
        let (tx, mut rx) = make_channel();
        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();

        line_tx.send("STAGE:DONE".to_string()).unwrap();
        drop(line_tx); // close sender so drain_lines can finish

        let mut total = 1024u64;
        let mut last = 0.0f32;
        drain_lines(&mut line_rx, &tx, &mut total, &mut last).await;

        let events = drain(&mut rx);
        assert!(
            events.iter().any(|e| matches!(e, FlashEvent::Completed)),
            "STAGE:DONE in drain should emit Completed"
        );
    }

    #[tokio::test]
    async fn drain_lines_error_sends_failed() {
        let (tx, mut rx) = make_channel();
        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();

        line_tx.send("ERROR:something broke".to_string()).unwrap();
        drop(line_tx);

        let mut total = 1024u64;
        let mut last = 0.0f32;
        drain_lines(&mut line_rx, &tx, &mut total, &mut last).await;

        let events = drain(&mut rx);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, FlashEvent::Failed(msg) if msg == "something broke")),
            "ERROR in drain should emit Failed"
        );
    }

    #[tokio::test]
    async fn drain_lines_log_forwarded() {
        let (tx, mut rx) = make_channel();
        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();

        line_tx.send("LOG:flushing caches".to_string()).unwrap();
        drop(line_tx);

        let mut total = 1024u64;
        let mut last = 0.0f32;
        drain_lines(&mut line_rx, &tx, &mut total, &mut last).await;

        let events = drain(&mut rx);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, FlashEvent::Log(m) if m == "flushing caches")),
            "LOG in drain should be forwarded"
        );
    }

    #[tokio::test]
    async fn drain_lines_progress_forwarded() {
        let (tx, mut rx) = make_channel();
        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();

        line_tx.send("PROGRESS:512000:8.0".to_string()).unwrap();
        drop(line_tx);

        let mut total = 1_024_000u64;
        let mut last = 0.0f32;
        drain_lines(&mut line_rx, &tx, &mut total, &mut last).await;

        let events = drain(&mut rx);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, FlashEvent::Progress(_, _, _))),
            "PROGRESS in drain should emit a Progress event"
        );
    }

    #[tokio::test]
    async fn drain_lines_empty_channel_emits_nothing() {
        let (tx, mut rx) = make_channel();
        let (_line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();
        // Do NOT send anything — just drop so the channel is empty.
        drop(_line_tx);

        let mut total = 1024u64;
        let mut last = 0.0f32;
        drain_lines(&mut line_rx, &tx, &mut total, &mut last).await;

        assert!(drain(&mut rx).is_empty());
    }

    // ── run_flash: validation guards ─────────────────────────────────────────

    #[tokio::test]
    async fn run_flash_missing_image_sends_failed() {
        let (tx, mut rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));

        run_flash(
            PathBuf::from("/nonexistent/image.iso"),
            PathBuf::from("/dev/null"),
            cancel,
            tx,
        )
        .await;

        let events = drain(&mut rx);
        assert!(
            events.iter().any(|e| matches!(e, FlashEvent::Failed(_))),
            "missing image must produce a Failed event"
        );
    }

    #[tokio::test]
    async fn run_flash_empty_image_sends_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let empty = tmp.path().join("empty.iso");
        std::fs::write(&empty, b"").unwrap(); // zero bytes

        let (tx, mut rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));

        run_flash(empty, PathBuf::from("/dev/null"), cancel, tx).await;

        let events = drain(&mut rx);
        assert!(
            events.iter().any(|e| matches!(e, FlashEvent::Failed(_))),
            "empty image must produce a Failed event"
        );
    }
}
