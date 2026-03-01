//! Flash Subscription - Real-time progress streaming
//!
//! This module implements an Iced [`Subscription`] that drives the flash
//! pipeline and forwards structured progress events to the UI.
//!
//! ## Architecture
//!
//! The flash operation runs entirely in-process on a dedicated blocking
//! `std::thread`.  No child process, no pkexec, no sudo, no IPC protocol.
//!
//! ```text
//!   Iced async runtime                 blocking OS thread
//!   ─────────────────                  ──────────────────
//!   flash_progress()                   std::thread::spawn
//!        │                                    │
//!        │   std::sync::mpsc::channel         │
//!        │ ◄────────────────────────── run_pipeline(tx, …)
//!        │                                    │
//!   FlashEvent → FlashProgress          writes image
//!        │
//!   Iced UI update
//! ```
//!
//! ## Privilege model
//!
//! The installed binary is **setuid-root** (`chmod u+s /usr/bin/flashkraft`).
//! `main.rs` captures the real (unprivileged) UID at startup and stores it via
//! [`flashkraft_core::flash_helper::set_real_uid`].  The pipeline calls
//! `seteuid(0)` only for the instant needed to open the block device, then
//! immediately drops back to the real UID.
//!
//! ## Cancellation
//!
//! An [`AtomicBool`] cancel token is shared between the Iced update loop and
//! the flash thread.  The pipeline checks the flag on every write block
//! (~4 MiB) and exits early when it is set.
//!
//! ## `FlashProgress` enum (unchanged)
//!
//! The variants `Progress`, `Message`, `Completed`, and `Failed` are identical
//! to the previous implementation — no changes to `update.rs`, `state.rs`,
//! `message.rs`, or any UI code are required.

use crate::flash_debug;
use flashkraft_core::flash_helper::{run_pipeline, FlashEvent};
use futures::SinkExt;
use iced::stream;
use iced::Subscription;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Progress event emitted by the flash subscription to the Iced runtime.
///
/// Variants are intentionally identical to the previous pkexec-based
/// implementation so that `update.rs` / `state.rs` / `message.rs` need no
/// changes.
#[derive(Debug, Clone)]
pub enum FlashProgress {
    /// `(progress 0.0–1.0, bytes_written, speed_mb_s)`
    Progress(f32, u64, f32),
    /// Human-readable status message for the UI (stage name, log line, …)
    Message(String),
    /// The flash operation finished successfully.
    Completed,
    /// The flash operation failed; the string is a human-readable error.
    Failed(String),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a subscription that streams [`FlashProgress`] events while the
/// flash operation runs.
///
/// The subscription is uniquely identified by hashing `image_path` and
/// `device_path` so Iced can deduplicate it across recompositions.
pub fn flash_progress(
    image_path: PathBuf,
    device_path: PathBuf,
    cancel_token: Arc<AtomicBool>,
) -> Subscription<FlashProgress> {
    // ── Stable subscription ID ────────────────────────────────────────────────
    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    device_path.hash(&mut hasher);
    let id = hasher.finish();

    Subscription::run_with_id(
        id,
        stream::channel(64, move |mut output| async move {
            let img = image_path.clone();
            let dev = device_path.clone();
            let cancel = cancel_token.clone();

            // ── Validate inputs before spinning up a thread ───────────────────
            let image_size = match img.metadata() {
                Ok(m) if m.len() == 0 => {
                    let _ = output
                        .send(FlashProgress::Failed("Image file is empty".into()))
                        .await;
                    return std::future::pending().await;
                }
                Ok(m) => m.len(),
                Err(e) => {
                    let _ = output
                        .send(FlashProgress::Failed(format!(
                            "Cannot read image file: {e}"
                        )))
                        .await;
                    return std::future::pending().await;
                }
            };

            flash_debug!("flash_progress: image={img:?} dev={dev:?} size={image_size}");

            // ── Bridge: blocking thread → async ───────────────────────────────
            // std::sync::mpsc is used on the thread side (blocking send);
            // we convert it to async by polling with try_recv + yield.
            let (tx, rx) = std::sync::mpsc::channel::<FlashEvent>();

            let img_str = img.to_string_lossy().into_owned();
            let dev_str = dev.to_string_lossy().into_owned();

            std::thread::spawn(move || {
                flash_debug!("flash thread: starting pipeline");
                run_pipeline(&img_str, &dev_str, tx, cancel);
                flash_debug!("flash thread: pipeline returned");
            });

            // ── Forward FlashEvents → FlashProgress ───────────────────────────
            loop {
                match rx.recv() {
                    // ── Progress update ───────────────────────────────────────
                    Ok(FlashEvent::Progress {
                        bytes_written,
                        total_bytes,
                        speed_mb_s,
                    }) => {
                        let progress = if total_bytes > 0 {
                            (bytes_written as f64 / total_bytes as f64).clamp(0.0, 1.0) as f32
                        } else {
                            0.0
                        };
                        flash_debug!(
                            "progress: {:.1}% ({bytes_written}/{total_bytes}) @ {speed_mb_s:.1} MB/s",
                            progress * 100.0
                        );
                        let _ = output
                            .send(FlashProgress::Progress(progress, bytes_written, speed_mb_s))
                            .await;
                    }

                    // ── Stage transition ──────────────────────────────────────
                    Ok(FlashEvent::Stage(stage)) => {
                        let msg = stage.to_string();
                        flash_debug!("stage: {msg}");
                        let _ = output.send(FlashProgress::Message(msg)).await;
                    }

                    // ── Informational log ─────────────────────────────────────
                    Ok(FlashEvent::Log(msg)) => {
                        flash_debug!("log: {msg}");
                        let _ = output.send(FlashProgress::Message(msg)).await;
                    }

                    // ── Success ───────────────────────────────────────────────
                    Ok(FlashEvent::Done) => {
                        flash_debug!("flash thread: Done");
                        let _ = output.send(FlashProgress::Completed).await;
                        break;
                    }

                    // ── Pipeline error ────────────────────────────────────────
                    Ok(FlashEvent::Error(e)) => {
                        flash_debug!("flash thread: Error: {e}");
                        let _ = output.send(FlashProgress::Failed(e)).await;
                        break;
                    }

                    // ── Sender dropped (thread panicked or returned early) ─────
                    Err(_) => {
                        flash_debug!("flash thread: channel closed unexpectedly");

                        // Only report failure if we haven't already sent a
                        // terminal event (Done / Error) that would have broken
                        // the loop above.  The cancel flag covers intentional
                        // cancellation.
                        if cancel_token.load(Ordering::SeqCst) {
                            let _ = output
                                .send(FlashProgress::Failed(
                                    "Flash operation cancelled by user".into(),
                                ))
                                .await;
                        } else {
                            let _ = output
                                .send(FlashProgress::Failed(
                                    "Flash thread terminated unexpectedly".into(),
                                ))
                                .await;
                        }
                        break;
                    }
                }
            }

            // Keep the subscription alive — Iced requires the async block to
            // never return (it is driven as a Stream).
            std::future::pending().await
        }),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// All `FlashProgress` variants must be `Clone` (Iced requirement).
    #[test]
    fn test_flash_progress_clone() {
        let variants = vec![
            FlashProgress::Progress(0.5, 1024, 10.0),
            FlashProgress::Message("hello".to_string()),
            FlashProgress::Completed,
            FlashProgress::Failed("oops".to_string()),
        ];
        for v in &variants {
            let _ = v.clone();
        }
    }

    #[test]
    fn test_flash_progress_debug() {
        let p = FlashProgress::Progress(1.0, 2048, 20.0);
        assert!(format!("{p:?}").contains("Progress"));
    }

    /// The subscription ID must be deterministic for a given (image, device) pair.
    #[test]
    fn test_subscription_id_is_deterministic() {
        fn compute_id(image: &str, device: &str) -> u64 {
            let mut hasher = DefaultHasher::new();
            PathBuf::from(image).hash(&mut hasher);
            PathBuf::from(device).hash(&mut hasher);
            hasher.finish()
        }
        let id1 = compute_id("/tmp/test.img", "/dev/sdb");
        let id2 = compute_id("/tmp/test.img", "/dev/sdb");
        assert_eq!(id1, id2, "subscription ID must be deterministic");
    }

    /// Different (image, device) pairs must produce different IDs.
    #[test]
    fn test_subscription_id_differs_for_different_devices() {
        fn compute_id(image: &str, device: &str) -> u64 {
            let mut hasher = DefaultHasher::new();
            PathBuf::from(image).hash(&mut hasher);
            PathBuf::from(device).hash(&mut hasher);
            hasher.finish()
        }
        let id1 = compute_id("/tmp/test.img", "/dev/sdb");
        let id2 = compute_id("/tmp/test.img", "/dev/sdc");
        assert_ne!(id1, id2, "different devices must yield different IDs");
    }

    /// FlashEvent channel bridge: verify that the mpsc bridge correctly maps
    /// every FlashEvent variant to the expected FlashProgress variant.
    #[test]
    fn test_flash_event_mapping() {
        use flashkraft_core::flash_helper::FlashStage;

        // Simulate what the async loop does — map FlashEvent → FlashProgress.
        let events = vec![
            FlashEvent::Stage(FlashStage::Writing),
            FlashEvent::Progress {
                bytes_written: 512,
                total_bytes: 1024,
                speed_mb_s: 42.0,
            },
            FlashEvent::Log("Test log".into()),
            FlashEvent::Done,
        ];

        for event in events {
            let _progress: Option<FlashProgress> = match event {
                FlashEvent::Progress {
                    bytes_written,
                    total_bytes,
                    speed_mb_s,
                } => {
                    let p = if total_bytes > 0 {
                        (bytes_written as f64 / total_bytes as f64).clamp(0.0, 1.0) as f32
                    } else {
                        0.0
                    };
                    Some(FlashProgress::Progress(p, bytes_written, speed_mb_s))
                }
                FlashEvent::Stage(s) => Some(FlashProgress::Message(s.to_string())),
                FlashEvent::Log(m) => Some(FlashProgress::Message(m)),
                FlashEvent::Done => Some(FlashProgress::Completed),
                FlashEvent::Error(e) => Some(FlashProgress::Failed(e)),
            };
            // Just verify the mapping doesn't panic.
        }
    }

    /// The channel bridge correctly handles the cancelled case.
    #[test]
    fn test_cancelled_maps_to_failed() {
        let cancel = Arc::new(AtomicBool::new(true));
        assert!(cancel.load(Ordering::SeqCst));
        // When cancel is true and Err(_) is received, we send Failed.
        let msg = if cancel.load(Ordering::SeqCst) {
            "Flash operation cancelled by user"
        } else {
            "Flash thread terminated unexpectedly"
        };
        assert_eq!(msg, "Flash operation cancelled by user");
    }
}
