//! Flash Writer Protocol Demo — flashkraft-core
//!
//! Simulates a complete flash operation output stream and feeds every line
//! through [`flashkraft_core::flash_writer::parse_script_line`], printing
//! each parsed [`ScriptLine`] variant in a human-readable form.
//!
//! This is the same parser used by both the GUI subscription and the TUI
//! flash-runner to interpret structured stdout from the privileged helper.
//!
//! # Wire protocol
//!
//! | Line prefix              | Parsed as            | Meaning                        |
//! |--------------------------|----------------------|--------------------------------|
//! | `STAGE:<name>`           | `Stage(FlashStage)`  | Pipeline stage transition      |
//! | `SIZE:<bytes>`           | `Size(u64)`          | Total image size in bytes      |
//! | `PROGRESS:<bytes>:<spd>` | `Progress(u64, f32)` | Write progress update          |
//! | `LOG:<message>`          | `Log(String)`        | Informational status message   |
//! | `ERROR:<message>`        | `Error(String)`      | Terminal error                 |
//! | `<dd-style line>`        | `DdProgress(...)`    | Legacy dd carriage-return line |
//! | `<number>+0 records out` | `DdExit(0)`          | dd clean exit                  |
//!
//! # Run
//!
//! ```bash
//! cargo run -p flashkraft-core --example flash_writer_demo
//! ```

use flashkraft_core::flash_writer::{parse_script_line, FlashStage, ScriptLine};

// ── Colour helpers (ANSI, no external dep) ───────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";
const BLUE: &str = "\x1b[34m";

fn main() {
    print_header();

    // ── Section 1: Rust helper protocol (nominal flash) ──────────────────────
    section("Rust helper — nominal flash (1 GiB image onto /dev/sdb)");

    let nominal: &[&str] = &[
        // Stage transitions
        "STAGE:UNMOUNTING",
        "LOG:Unmounting /dev/sdb1 (MNT_DETACH)…",
        "LOG:Unmounting /dev/sdb2 (MNT_DETACH)…",
        "LOG:All partitions detached.",
        "STAGE:WRITING",
        // Size announcement
        "SIZE:1073741824",
        // Progress updates (bytes written, speed MB/s)
        "PROGRESS:67108864:28.4",
        "PROGRESS:134217728:31.7",
        "PROGRESS:268435456:33.1",
        "PROGRESS:536870912:34.9",
        "PROGRESS:805306368:35.2",
        "PROGRESS:1073741824:34.8",
        // Sync
        "STAGE:SYNCING",
        "LOG:fsync complete. Calling sync()…",
        // Partition table re-read
        "STAGE:REREADING",
        "LOG:BLKRRPART ioctl succeeded — kernel sees new partition table.",
        // Verification
        "STAGE:VERIFYING",
        "LOG:SHA-256(image) = a3f1c9d2e7b8041556fabe23cd90147738ccaef1d5bcf402910e8da672f193e",
        "LOG:SHA-256(device[0..1073741824]) = a3f1c9d2e7b8041556fabe23cd90147738ccaef1d5bcf402910e8da672f193e",
        "LOG:Verification passed ✓",
        // Done
        "STAGE:DONE",
    ];

    run_simulation(nominal);

    // ── Section 2: Rust helper protocol (error path) ─────────────────────────
    section("Rust helper — error path (device disappears mid-write)");

    let error_path: &[&str] = &[
        "STAGE:UNMOUNTING",
        "LOG:Unmounting /dev/sdb1 (MNT_DETACH)…",
        "STAGE:WRITING",
        "SIZE:2147483648",
        "PROGRESS:67108864:30.0",
        "PROGRESS:134217728:31.2",
        "ERROR:Write failed after 134217728 bytes: No space left on device (os error 28)",
    ];

    run_simulation(error_path);

    // ── Section 3: Legacy dd output ──────────────────────────────────────────
    section("Legacy dd output (carriage-return-delimited progress)");

    let dd_lines: &[&str] = &[
        // These are the \r-terminated lines dd emits to stderr / stdout
        "67108864 bytes (67 MB, 64 MiB) copied, 2.1 s, 31.5 MB/s",
        "268435456 bytes (268 MB, 256 MiB) copied, 8.3 s, 32.1 MB/s",
        "536870912 bytes (537 MB, 512 MiB) copied, 16.1 s, 33.3 MB/s",
        "1073741824 bytes (1.1 GB, 1.0 GiB) copied, 30.9 s, 34.7 MB/s",
        // dd clean exit line
        "2097152+0 records in",
        "2097152+0 records out",
    ];

    run_simulation(dd_lines);

    // ── Section 4: FlashStage Display ────────────────────────────────────────
    section("FlashStage::Display round-trip");

    let stages = [
        FlashStage::Starting,
        FlashStage::Unmounting,
        FlashStage::Writing,
        FlashStage::Syncing,
        FlashStage::Rereading,
        FlashStage::Verifying,
        FlashStage::Done,
        FlashStage::Failed("Simulated write error".to_string()),
    ];

    // Wire-format keys that the helper actually emits for each stage.
    // Starting and Failed have no dedicated STAGE: wire key (Starting is the
    // implicit initial state; Failed arrives via ERROR: lines).
    let wire_keys: &[Option<&str>] = &[
        None, // Starting  — no wire key
        Some("UNMOUNTING"),
        Some("WRITING"),
        Some("SYNCING"),
        Some("REREADING"),
        Some("VERIFYING"),
        Some("DONE"),
        None, // Failed    — no wire key (comes via ERROR:)
    ];

    for (stage, wire_key) in stages.iter().zip(wire_keys.iter()) {
        let display = stage.to_string();

        // Test the round-trip through the actual wire key (where one exists).
        let round_trip = if let Some(key) = wire_key {
            let line = format!("STAGE:{key}");
            let parsed = parse_script_line(&line);
            let ok = matches!(&parsed, ScriptLine::Stage(s) if *s == *stage);
            if ok {
                format!("{GREEN}✓ ok{RESET}")
            } else {
                format!("{RED}✗ mismatch{RESET}")
            }
        } else {
            format!("{DIM}n/a{RESET}")
        };

        let wire_str = wire_key
            .map(|k| format!("{CYAN}STAGE:{k}{RESET}"))
            .unwrap_or_else(|| format!("{DIM}(none){RESET}"));

        println!("  {BOLD}{display:<30}{RESET}  wire={wire_str:<50}  round-trip={round_trip}",);
    }

    // ── Section 5: Edge cases ────────────────────────────────────────────────
    section("Edge cases & unknown lines");

    let edge: &[&str] = &[
        // Leading/trailing whitespace
        "  STAGE:WRITING  ",
        // Unknown prefix — should parse as Unknown
        "some random garbage line",
        // Empty string
        "",
        // LOG with colon in the message
        "LOG:Path is: /home/user/ubuntu-24.04.iso",
        // PROGRESS with zero speed (stalled write)
        "PROGRESS:1048576:0.0",
        // SIZE of zero (degenerate)
        "SIZE:0",
    ];

    run_simulation(edge);

    println!();
    println!("{BOLD}Demo complete.{RESET}");
    println!(
        "{DIM}All lines above were processed by \
         flashkraft_core::flash_writer::parse_script_line.{RESET}"
    );
    println!();
}

// ── Simulation driver ─────────────────────────────────────────────────────────

fn run_simulation(lines: &[&str]) {
    for (i, &raw) in lines.iter().enumerate() {
        let parsed = parse_script_line(raw);
        let prefix = format!("{DIM}{:>3}.{RESET}", i + 1);
        println!("{prefix} {}", format_raw(raw));
        println!("     {}", format_parsed(&parsed));
        println!();
    }
}

// ── Formatting helpers ────────────────────────────────────────────────────────

fn format_raw(raw: &str) -> String {
    if raw.is_empty() {
        format!("{DIM}(empty line){RESET}")
    } else {
        format!("{DIM}← {raw:?}{RESET}")
    }
}

fn format_parsed(line: &ScriptLine) -> String {
    match line {
        ScriptLine::Stage(stage) => {
            format!("{GREEN}{BOLD}Stage{RESET}({CYAN}{stage}{RESET})")
        }
        ScriptLine::Size(bytes) => {
            format!(
                "{GREEN}{BOLD}Size{RESET}({CYAN}{bytes}{RESET} bytes = {:.2} MiB)",
                *bytes as f64 / 1_048_576.0
            )
        }
        ScriptLine::Progress(bytes, speed) => {
            format!(
                "{GREEN}{BOLD}Progress{RESET}(written={CYAN}{:.1} MiB{RESET}, \
                 speed={CYAN}{speed:.1} MB/s{RESET})",
                *bytes as f64 / 1_048_576.0
            )
        }
        ScriptLine::DdProgress(bytes, speed) => {
            format!(
                "{BLUE}{BOLD}DdProgress{RESET}(written={CYAN}{:.1} MiB{RESET}, \
                 speed={CYAN}{speed:.1} MB/s{RESET})",
                *bytes as f64 / 1_048_576.0
            )
        }
        ScriptLine::DdExit(code) => {
            let colour = if *code == 0 { GREEN } else { RED };
            format!("{BLUE}{BOLD}DdExit{RESET}(code={colour}{code}{RESET})")
        }
        ScriptLine::Log(msg) => {
            format!("{YELLOW}{BOLD}Log{RESET}({DIM}\"{msg}\"{RESET})")
        }
        ScriptLine::Error(msg) => {
            format!("{RED}{BOLD}Error{RESET}({RED}\"{msg}\"{RESET})")
        }
        ScriptLine::Unknown(raw) => {
            format!("{MAGENTA}{BOLD}Unknown{RESET}({DIM}\"{raw}\"{RESET})")
        }
    }
}

// ── UI chrome ─────────────────────────────────────────────────────────────────

fn print_header() {
    println!();
    println!("{BOLD}{CYAN}┌─────────────────────────────────────────────────────┐{RESET}");
    println!(
        "{BOLD}{CYAN}│{RESET}  {BOLD}flashkraft-core  ·  Flash Writer Protocol Demo{RESET}   \
         {BOLD}{CYAN}│{RESET}"
    );
    println!("{BOLD}{CYAN}└─────────────────────────────────────────────────────┘{RESET}");
    println!();
    println!(
        "Every line is parsed by {CYAN}parse_script_line(){RESET} \
         and printed with its variant."
    );
    println!();
}

fn section(title: &str) {
    let pad = "─".repeat(60usize.saturating_sub(title.len() + 4));
    println!("{BOLD}── {title} {DIM}{pad}{RESET}");
    println!();
}
