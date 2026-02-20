//! Flash Writer - Protocol types and parsing for the pure-Rust flash pipeline
//!
//! This module provides the shared vocabulary between the privileged flash
//! helper ([`crate::core::flash_helper`]) and the Iced subscription
//! ([`crate::core::flash_subscription`]) that drives the UI.
//!
//! ## Responsibilities
//!
//! | Item | Purpose |
//! |------|---------|
//! | [`FlashStage`] | Strongly-typed enum of the five pipeline stages |
//! | [`ScriptLine`] | Parsed line variants emitted by the helper process |
//! | [`DdProgress`] | Structured progress from a parsed `dd status=progress` line (legacy) |
//! | [`parse_script_line`] | Parse one line of helper stdout into a [`ScriptLine`] |
//! | [`parse_dd_progress`] | Parse a raw `dd` progress line (legacy / compat) |
//! | [`parse_speed_to_mb_s`] | Normalise any speed unit (`kB/s`, `MB/s`, `GB/s`, …) to MB/s |
//!
//! ## Wire protocol
//!
//! The privileged helper (`flashkraft --flash-helper <image> <device>`) writes
//! one structured line per event to its stdout.  The subscription reads those
//! lines and calls [`parse_script_line`] on each one:
//!
//! ```text
//! STAGE:UNMOUNTING
//! LOG:Unmounting /dev/sdb1
//! SIZE:7948206080
//! STAGE:WRITING
//! PROGRESS:4194304:35.60        ← bytes_written:speed_mb_s
//! PROGRESS:7948206080:38.20
//! STAGE:SYNCING
//! LOG:Kernel write-back caches flushed
//! STAGE:REREADING
//! LOG:Kernel partition table refreshed
//! STAGE:VERIFYING
//! LOG:Verification passed (a3f1…)
//! STAGE:DONE
//! ```
//!
//! On any failure the helper emits `ERROR:<message>` and exits non-zero.
//!
//! All pure functions in this module are fully unit-tested.

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A parsed and normalised progress update from a legacy `dd status=progress` line.
///
/// The primary progress format is now [`ScriptLine::Progress`]
/// (`PROGRESS:<bytes>:<speed_mb_s>`), emitted directly by the pure-Rust
/// helper.  [`DdProgress`] is retained for backward compatibility with any
/// tooling that still pipes raw `dd` output.
#[derive(Debug, Clone, PartialEq)]
pub struct DdProgress {
    /// Bytes written so far.
    pub bytes_written: u64,
    /// Transfer speed in MB/s (always normalised, regardless of the unit `dd` emitted).
    pub speed_mb_s: f32,
    /// Progress ratio in `[0.0, 1.0]`.
    pub progress: f32,
}

/// The stage currently active in the five-step flash pipeline.
///
/// The helper emits `STAGE:<name>` lines that are parsed into this enum by
/// [`parse_script_line`].  The UI can use [`FlashStage::to_string`] directly
/// as a human-readable status message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashStage {
    /// Initial state before the pipeline starts.
    Starting,
    /// All partitions of the target device are being lazily unmounted.
    Unmounting,
    /// The image is being written to the block device in 4 MiB chunks.
    Writing,
    /// `fsync` + `sync` are called to flush all kernel write-back caches.
    Syncing,
    /// `BLKRRPART` ioctl asks the kernel to re-read the partition table.
    Rereading,
    /// SHA-256 of the source image is compared against a read-back of the device.
    Verifying,
    /// The entire pipeline completed successfully.
    Done,
    /// The pipeline failed with the enclosed error message.
    Failed(String),
}

impl std::fmt::Display for FlashStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlashStage::Starting => write!(f, "Starting…"),
            FlashStage::Unmounting => write!(f, "Unmounting partitions…"),
            FlashStage::Writing => write!(f, "Writing image to device…"),
            FlashStage::Syncing => write!(f, "Flushing write buffers…"),
            FlashStage::Rereading => write!(f, "Refreshing partition table…"),
            FlashStage::Verifying => write!(f, "Verifying written data…"),
            FlashStage::Done => write!(f, "Flash complete!"),
            FlashStage::Failed(msg) => write!(f, "Failed: {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Speed parsing
// ---------------------------------------------------------------------------

/// Normalise a speed value + unit token to MB/s.
///
/// `dd` (and occasionally other tools) emit speeds with varying units:
/// `"kB/s"`, `"MB/s"`, `"GB/s"`, `"MiB/s"`, `"GiB/s"`, `"B/s"`, etc.
/// This function converts all of them to a consistent MB/s `f32`.
///
/// The pure-Rust helper emits speeds already in MB/s via the structured
/// `PROGRESS:<bytes>:<speed_mb_s>` line, so this function is mainly used
/// when parsing legacy `dd status=progress` output.
///
/// Returns `0.0` when either argument cannot be interpreted.
pub fn parse_speed_to_mb_s(value_str: &str, unit_str: &str) -> f32 {
    let value: f32 = match value_str.replace(',', ".").trim().parse() {
        Ok(v) => v,
        Err(_) => return 0.0,
    };

    // Normalise unit (lower-case for comparison)
    let unit = unit_str.trim().to_lowercase();

    // Strip a trailing "/s" or "ps" so we only compare the prefix
    let base = unit
        .strip_suffix("/s")
        .or_else(|| unit.strip_suffix("ps"))
        .unwrap_or(&unit);

    match base {
        "kb" | "kib" => value / 1024.0,
        "mb" | "mib" => value,
        "gb" | "gib" => value * 1024.0,
        "tb" | "tib" => value * 1024.0 * 1024.0,
        "b" => value / (1024.0 * 1024.0),
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// `dd` progress line parsing
// ---------------------------------------------------------------------------

/// Parse a single `dd status=progress` line into a [`DdProgress`].
///
/// `dd` writes lines (separated by `\r` while running, `\n` at the end)
/// in this format:
///
/// ```text
/// 1073741824 bytes (1.1 GB, 1.0 GiB) copied, 30.2 s, 35.6 MB/s
/// ```
///
/// Returns `None` if `line` does not look like a dd progress line, or if
/// `image_size` is zero (can't compute a ratio).
pub fn parse_dd_progress(line: &str, image_size: u64) -> Option<DdProgress> {
    let line = line.trim();

    // Must contain both "bytes" and "copied" to be a dd progress line.
    if !line.contains("bytes") || !line.contains("copied") {
        return None;
    }

    // First token is the byte count.
    let mut tokens = line.split_whitespace();
    let bytes_written: u64 = tokens.next()?.parse().ok()?;

    // Last two tokens are "<value> <unit/s>" e.g. "35.6 MB/s"
    let all_tokens: Vec<&str> = line.split_whitespace().collect();
    if all_tokens.len() < 2 {
        return None;
    }

    let unit_str = all_tokens[all_tokens.len() - 1]; // e.g. "MB/s"
    let value_str = all_tokens[all_tokens.len() - 2]; // e.g. "35.6"

    let speed_mb_s = parse_speed_to_mb_s(value_str, unit_str);

    let progress = if image_size > 0 {
        (bytes_written as f64 / image_size as f64).clamp(0.0, 1.0) as f32
    } else {
        return None;
    };

    Some(DdProgress {
        bytes_written,
        speed_mb_s,
        progress,
    })
}

// ---------------------------------------------------------------------------
// Script output line parsing
// ---------------------------------------------------------------------------

/// Parse a single line of output from the flash helper / script.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptLine {
    /// `STAGE:<name>` — stage transition
    Stage(FlashStage),
    /// `SIZE:<bytes>` — total image size (needed for progress ratio)
    Size(u64),
    /// `PROGRESS:<bytes_written>:<speed_mb_s>` — structured progress from
    /// the pure-Rust helper (`flash_helper.rs`).  This is the primary format.
    Progress(u64, f32),
    /// `DD_EXIT:<code>` — exit code of a `dd` invocation (legacy / fallback)
    DdExit(i32),
    /// `ERROR:<message>` — explicit error (terminal; process exits non-zero)
    Error(String),
    /// `LOG:<message>` — informational message
    Log(String),
    /// A raw `dd status=progress` line embedded in the output stream (legacy)
    DdProgress(u64, f32), // (bytes_written, speed_mb_s) — caller computes ratio
    /// Anything we do not recognise
    Unknown(String),
}

/// Parse one line of output from the pure-Rust flash helper (or the legacy
/// bash script fallback).
///
/// Lines are matched in priority order:
/// 1. `PROGRESS:<bytes>:<speed_mb_s>` — structured progress (Rust helper)
/// 2. `STAGE:<name>` — stage transition
/// 3. `SIZE:<bytes>` — total image size
/// 4. `DD_EXIT:<code>` — dd exit code (legacy)
/// 5. `ERROR:<message>` — terminal error
/// 6. `LOG:<message>` — informational message
/// 7. dd `status=progress` line (legacy, contains "bytes" and "copied")
/// 8. `Unknown` — anything else
pub fn parse_script_line(line: &str) -> ScriptLine {
    let line = line.trim();

    // ── PROGRESS:<bytes>:<speed_mb_s> (pure-Rust helper) ────────────────────
    if let Some(rest) = line.strip_prefix("PROGRESS:") {
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() == 2 {
            if let (Ok(bytes), Ok(speed)) = (
                parts[0].trim().parse::<u64>(),
                parts[1].trim().parse::<f32>(),
            ) {
                return ScriptLine::Progress(bytes, speed);
            }
        }
    }

    if let Some(rest) = line.strip_prefix("STAGE:") {
        let stage = match rest.trim() {
            "UNMOUNTING" => FlashStage::Unmounting,
            "WRITING" => FlashStage::Writing,
            "SYNCING" => FlashStage::Syncing,
            "REREADING" => FlashStage::Rereading,
            "VERIFYING" => FlashStage::Verifying,
            "DONE" => FlashStage::Done,
            other => FlashStage::Failed(other.to_string()),
        };
        return ScriptLine::Stage(stage);
    }

    if let Some(rest) = line.strip_prefix("SIZE:") {
        if let Ok(n) = rest.trim().parse::<u64>() {
            return ScriptLine::Size(n);
        }
    }

    if let Some(rest) = line.strip_prefix("DD_EXIT:") {
        if let Ok(code) = rest.trim().parse::<i32>() {
            return ScriptLine::DdExit(code);
        }
    }

    if let Some(rest) = line.strip_prefix("ERROR:") {
        return ScriptLine::Error(rest.trim().to_string());
    }

    if let Some(rest) = line.strip_prefix("LOG:") {
        return ScriptLine::Log(rest.trim().to_string());
    }

    // Try to parse as a dd progress line (contains "bytes" and "copied")
    if line.contains("bytes") && line.contains("copied") {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() >= 2 {
            if let Ok(bytes) = tokens[0].parse::<u64>() {
                let unit_str = tokens[tokens.len() - 1];
                let value_str = tokens[tokens.len() - 2];
                let speed_mb_s = parse_speed_to_mb_s(value_str, unit_str);
                return ScriptLine::DdProgress(bytes, speed_mb_s);
            }
        }
    }

    ScriptLine::Unknown(line.to_string())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_speed_to_mb_s ─────────────────────────────────────────────────

    #[test]
    fn test_parse_speed_mb_s() {
        let speed = parse_speed_to_mb_s("35.6", "MB/s");
        assert!((speed - 35.6).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_kb_s() {
        // 1024 kB/s == 1 MB/s
        let speed = parse_speed_to_mb_s("1024", "kB/s");
        assert!((speed - 1.0).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_gb_s() {
        // 1 GB/s == 1024 MB/s
        let speed = parse_speed_to_mb_s("1", "GB/s");
        assert!((speed - 1024.0).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_mib_s() {
        // MiB/s ≈ MB/s (same treatment)
        let speed = parse_speed_to_mb_s("10.0", "MiB/s");
        assert!((speed - 10.0).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_kib_s() {
        let speed = parse_speed_to_mb_s("512", "kiB/s");
        assert!((speed - 0.5).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_bytes_per_s() {
        let speed = parse_speed_to_mb_s("1048576", "B/s");
        assert!((speed - 1.0).abs() < 0.01, "got {speed}");
    }

    #[test]
    fn test_parse_speed_invalid() {
        assert_eq!(parse_speed_to_mb_s("abc", "MB/s"), 0.0);
        assert_eq!(parse_speed_to_mb_s("10", ""), 0.0);
        assert_eq!(parse_speed_to_mb_s("", "MB/s"), 0.0);
    }

    #[test]
    fn test_parse_speed_comma_decimal() {
        // Some locales use comma as decimal separator
        let speed = parse_speed_to_mb_s("35,6", "MB/s");
        assert!((speed - 35.6).abs() < 0.01, "got {speed}");
    }

    // ── parse_dd_progress ───────────────────────────────────────────────────

    const IMAGE_1G: u64 = 1_073_741_824; // 1 GiB

    #[test]
    fn test_parse_dd_progress_mb_s() {
        let line = "1073741824 bytes (1.1 GB, 1.0 GiB) copied, 30.2 s, 35.6 MB/s";
        let p = parse_dd_progress(line, IMAGE_1G).expect("should parse");
        assert_eq!(p.bytes_written, IMAGE_1G);
        assert!((p.speed_mb_s - 35.6).abs() < 0.1, "speed={}", p.speed_mb_s);
        assert!((p.progress - 1.0).abs() < 0.001, "progress={}", p.progress);
    }

    #[test]
    fn test_parse_dd_progress_half() {
        let half = IMAGE_1G / 2;
        let line = format!("{half} bytes (537 MB, 512 MiB) copied, 15.1 s, 33.0 MB/s");
        let p = parse_dd_progress(&line, IMAGE_1G).expect("should parse");
        assert_eq!(p.bytes_written, half);
        assert!((p.progress - 0.5).abs() < 0.001, "progress={}", p.progress);
        assert!((p.speed_mb_s - 33.0).abs() < 0.1, "speed={}", p.speed_mb_s);
    }

    #[test]
    fn test_parse_dd_progress_kb_s() {
        // Early in the transfer dd sometimes reports in kB/s
        let line = "4096 bytes (4.1 kB, 4.0 KiB) copied, 0.001 s, 3.5 MB/s";
        let p = parse_dd_progress(line, IMAGE_1G).expect("should parse");
        assert_eq!(p.bytes_written, 4096);
        assert!((p.speed_mb_s - 3.5).abs() < 0.1, "speed={}", p.speed_mb_s);
    }

    #[test]
    fn test_parse_dd_progress_slow_kb_s() {
        // 500 kB/s = 0.488 MB/s
        let line = "4194304 bytes (4.2 MB, 4.0 MiB) copied, 8.4 s, 500 kB/s";
        let p = parse_dd_progress(line, IMAGE_1G).expect("should parse");
        assert!(
            (p.speed_mb_s - 500.0 / 1024.0).abs() < 0.01,
            "speed={}",
            p.speed_mb_s
        );
    }

    #[test]
    fn test_parse_dd_progress_not_a_progress_line() {
        assert!(parse_dd_progress("STAGE:WRITING", IMAGE_1G).is_none());
        assert!(parse_dd_progress("LOG:some message", IMAGE_1G).is_none());
        assert!(parse_dd_progress("", IMAGE_1G).is_none());
        assert!(parse_dd_progress("hello world", IMAGE_1G).is_none());
    }

    #[test]
    fn test_parse_dd_progress_zero_image_size() {
        let line = "1073741824 bytes (1.1 GB, 1.0 GiB) copied, 30.2 s, 35.6 MB/s";
        assert!(parse_dd_progress(line, 0).is_none());
    }

    #[test]
    fn test_parse_dd_progress_clamps_to_one() {
        // bytes_written > image_size (e.g. image is a compressed stream)
        let line = "2000000000 bytes (2.0 GB, 1.9 GiB) copied, 30.2 s, 35.6 MB/s";
        let p = parse_dd_progress(line, IMAGE_1G).expect("should parse");
        assert!(p.progress <= 1.0, "progress={}", p.progress);
        assert!(p.progress >= 0.0, "progress={}", p.progress);
    }

    // ── parse_script_line: PROGRESS (Rust helper format) ────────────────────

    #[test]
    fn test_parse_script_line_progress_mb_s() {
        match parse_script_line("PROGRESS:104857600:38.50") {
            ScriptLine::Progress(bytes, speed) => {
                assert_eq!(bytes, 104_857_600);
                assert!((speed - 38.5).abs() < 0.01, "speed={speed}");
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_script_line_progress_at_start() {
        // Very first progress update — bytes > 0, small speed
        match parse_script_line("PROGRESS:4194304:5.10") {
            ScriptLine::Progress(bytes, speed) => {
                assert_eq!(bytes, 4_194_304);
                assert!((speed - 5.1).abs() < 0.01, "speed={speed}");
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_script_line_progress_zero_speed() {
        match parse_script_line("PROGRESS:1024:0.00") {
            ScriptLine::Progress(bytes, speed) => {
                assert_eq!(bytes, 1024);
                assert_eq!(speed, 0.0);
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_script_line_progress_large_image() {
        // 7.4 GiB USB stick
        match parse_script_line("PROGRESS:7948206080:42.00") {
            ScriptLine::Progress(bytes, speed) => {
                assert_eq!(bytes, 7_948_206_080);
                assert!((speed - 42.0).abs() < 0.01, "speed={speed}");
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_script_line_progress_malformed_falls_through() {
        // Missing speed part — should not parse as Progress
        match parse_script_line("PROGRESS:1234567") {
            ScriptLine::Unknown(_) => {}
            other => panic!("expected Unknown for malformed PROGRESS, got {other:?}"),
        }
    }

    // ── parse_script_line: STAGE ─────────────────────────────────────────────

    #[test]
    fn test_parse_script_line_stage_writing() {
        assert_eq!(
            parse_script_line("STAGE:WRITING"),
            ScriptLine::Stage(FlashStage::Writing)
        );
    }

    #[test]
    fn test_parse_script_line_all_stages() {
        let cases = vec![
            ("STAGE:UNMOUNTING", FlashStage::Unmounting),
            ("STAGE:WRITING", FlashStage::Writing),
            ("STAGE:SYNCING", FlashStage::Syncing),
            ("STAGE:REREADING", FlashStage::Rereading),
            ("STAGE:VERIFYING", FlashStage::Verifying),
            ("STAGE:DONE", FlashStage::Done),
        ];
        for (input, expected) in cases {
            assert_eq!(
                parse_script_line(input),
                ScriptLine::Stage(expected.clone()),
                "input={input}"
            );
        }
    }

    #[test]
    fn test_parse_script_line_size() {
        assert_eq!(
            parse_script_line("SIZE:1073741824"),
            ScriptLine::Size(1_073_741_824)
        );
    }

    #[test]
    fn test_parse_script_line_dd_exit_zero() {
        assert_eq!(parse_script_line("DD_EXIT:0"), ScriptLine::DdExit(0));
    }

    #[test]
    fn test_parse_script_line_dd_exit_nonzero() {
        assert_eq!(parse_script_line("DD_EXIT:1"), ScriptLine::DdExit(1));
    }

    #[test]
    fn test_parse_script_line_error() {
        assert_eq!(
            parse_script_line("ERROR:Device not found: /dev/sdb"),
            ScriptLine::Error("Device not found: /dev/sdb".to_string())
        );
    }

    #[test]
    fn test_parse_script_line_log() {
        assert_eq!(
            parse_script_line("LOG:Unmounting /dev/sdb1"),
            ScriptLine::Log("Unmounting /dev/sdb1".to_string())
        );
    }

    #[test]
    fn test_parse_script_line_dd_progress() {
        // Legacy dd output format (still supported for backward compat)
        let line = "536870912 bytes (537 MB, 512 MiB) copied, 15.1 s, 33.0 MB/s";
        match parse_script_line(line) {
            ScriptLine::DdProgress(bytes, speed) => {
                assert_eq!(bytes, 536_870_912);
                assert!((speed - 33.0).abs() < 0.1, "speed={speed}");
            }
            other => panic!("expected DdProgress, got {other:?}"),
        }
    }

    /// PROGRESS: lines must take priority over any other interpretation.
    #[test]
    fn test_progress_takes_priority_over_unknown() {
        // A well-formed PROGRESS line must never fall through to Unknown.
        let line = "PROGRESS:536870912:33.00";
        assert!(
            matches!(parse_script_line(line), ScriptLine::Progress(_, _)),
            "PROGRESS: line must parse as ScriptLine::Progress"
        );
    }

    #[test]
    fn test_parse_script_line_unknown() {
        assert_eq!(
            parse_script_line("some random output"),
            ScriptLine::Unknown("some random output".to_string())
        );
    }

    #[test]
    fn test_parse_script_line_trims_whitespace() {
        assert_eq!(
            parse_script_line("  STAGE:DONE  "),
            ScriptLine::Stage(FlashStage::Done)
        );
    }

    // ── FlashStage Display ──────────────────────────────────────────────────

    #[test]
    fn test_flash_stage_display() {
        assert_eq!(FlashStage::Done.to_string(), "Flash complete!");
        assert!(FlashStage::Writing.to_string().contains("Writing"));
        assert!(FlashStage::Verifying.to_string().contains("Verif"));
        let failed = FlashStage::Failed("oops".to_string());
        assert!(failed.to_string().contains("oops"));
    }

    // ── Integration: full script output simulation ──────────────────────────

    /// Simulate the ordered lines a successful flash emits from the pure-Rust
    /// helper and verify that the parser handles them all correctly.
    #[test]
    fn test_full_script_output_simulation() {
        let image_size: u64 = 1_073_741_824;

        let simulated_output = vec![
            "STAGE:UNMOUNTING",
            "LOG:Unmounting /dev/sdb1",
            "SIZE:1073741824",
            "STAGE:WRITING",
            "LOG:Writing 1073741824 bytes from /path/to/image.iso → /dev/sdb",
            // Pure-Rust helper PROGRESS lines
            "PROGRESS:536870912:33.00",
            "PROGRESS:1073741824:35.60",
            "STAGE:SYNCING",
            "LOG:Kernel write-back caches flushed",
            "STAGE:REREADING",
            "LOG:Kernel partition table refreshed",
            "STAGE:VERIFYING",
            "LOG:Computing SHA-256 of source image",
            "LOG:Reading back 1073741824 bytes from device for verification",
            "LOG:Verification passed (abc123)",
            "STAGE:DONE",
        ];

        let mut stages_seen = Vec::new();
        let mut last_bytes = 0u64;
        let mut total_size_seen = 0u64;

        for line in simulated_output {
            match parse_script_line(line) {
                ScriptLine::Stage(s) => stages_seen.push(s),
                ScriptLine::Size(n) => total_size_seen = n,
                ScriptLine::Progress(bytes, speed) => {
                    assert!(bytes > 0, "bytes should be positive");
                    assert!(speed > 0.0, "speed should be positive");
                    last_bytes = bytes;
                }
                ScriptLine::Error(e) => panic!("unexpected error: {e}"),
                ScriptLine::Log(_)
                | ScriptLine::Unknown(_)
                | ScriptLine::DdProgress(_, _)
                | ScriptLine::DdExit(_) => {}
            }
        }

        assert_eq!(total_size_seen, image_size);
        assert_eq!(last_bytes, image_size);

        let expected_stages = vec![
            FlashStage::Unmounting,
            FlashStage::Writing,
            FlashStage::Syncing,
            FlashStage::Rereading,
            FlashStage::Verifying,
            FlashStage::Done,
        ];
        assert_eq!(stages_seen, expected_stages);
    }

    /// Simulate a write failure (device full / permission denied).
    #[test]
    fn test_failed_flash_output_simulation() {
        let simulated_output = vec![
            "STAGE:UNMOUNTING",
            "LOG:No mounted partitions found",
            "SIZE:1073741824",
            "STAGE:WRITING",
            "LOG:Writing 1073741824 bytes from /path/image.iso → /dev/sdb",
            "PROGRESS:4194304:3.80",
            "ERROR:Write error on device: No space left on device",
        ];

        let mut got_error = false;
        let mut error_msg = String::new();
        let mut last_progress_bytes = 0u64;

        for line in simulated_output {
            match parse_script_line(line) {
                ScriptLine::Progress(bytes, _) => last_progress_bytes = bytes,
                ScriptLine::Error(msg) => {
                    got_error = true;
                    error_msg = msg;
                }
                _ => {}
            }
        }

        assert_eq!(last_progress_bytes, 4_194_304);
        assert!(got_error, "should detect the ERROR: line");
        assert!(error_msg.contains("Write error"), "msg={error_msg}");
    }

    /// Simulate a verification failure from the Rust helper.
    #[test]
    fn test_verification_failure_simulation() {
        let simulated_output = vec![
            "SIZE:1073741824",
            "STAGE:WRITING",
            "PROGRESS:1073741824:35.60",
            "STAGE:SYNCING",
            "STAGE:REREADING",
            "STAGE:VERIFYING",
            "ERROR:Verification failed — data mismatch (image=aaa device=bbb)",
        ];

        let mut got_error = false;
        let mut error_msg = String::new();

        for line in simulated_output {
            if let ScriptLine::Error(msg) = parse_script_line(line) {
                got_error = true;
                error_msg = msg;
            }
        }

        assert!(got_error);
        assert!(error_msg.contains("Verification failed"));
        assert!(error_msg.contains("image=aaa"));
        assert!(error_msg.contains("device=bbb"));
    }
}
