//! Flash Pipeline
//!
//! Implements the entire privileged flash pipeline in-process.
//!
//! ## Privilege model
//!
//! The installed binary carries the **setuid-root** bit
//! (`sudo chmod u+s /usr/bin/flashkraft`).  At process startup `main.rs`
//! calls [`set_real_uid`] to record the unprivileged user's UID.
//!
//! When the pipeline needs to open a raw block device it temporarily
//! escalates to root via `nix::unistd::seteuid(0)`, opens the file
//! descriptor, then immediately drops back to the real UID.  Root is held
//! for less than one millisecond.
//!
//! ## Progress reporting
//!
//! The pipeline runs on a dedicated blocking thread spawned by the flash
//! subscription.  Progress is reported by sending [`FlashEvent`] values
//! through a [`std::sync::mpsc::Sender`] — no child process, no stdout
//! parsing, no IPC protocol.
//!
//! ## Pipeline stages
//!
//! 1. Validate inputs (image exists and is non-empty, device exists, not a partition node)
//! 2. Unmount all partitions of the target device (lazy / `MNT_DETACH`)
//! 3. Write the image in 4 MiB blocks, reporting progress every 400 ms
//! 4. `fsync` the device fd (hard error on failure)
//! 5. `fdatasync` + global `sync()` (belt-and-suspenders)
//! 6. `BLKRRPART` ioctl — ask the kernel to re-read the partition table
//! 7. SHA-256 verify: hash the source image, hash the first N bytes of the device, compare

#[cfg(unix)]
use nix::libc;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, OnceLock,
};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Write / read-back buffer: 4 MiB is a sweet spot for USB throughput.
const BLOCK_SIZE: usize = 4 * 1024 * 1024;

/// Minimum interval between `FlashEvent::Progress` emissions.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(400);

// ---------------------------------------------------------------------------
// Real-UID registry
// ---------------------------------------------------------------------------

/// The unprivileged UID of the user who launched the process.
///
/// Captured in `main.rs` via `nix::unistd::getuid()` before any `seteuid`
/// call and stored here via [`set_real_uid`].
static REAL_UID: OnceLock<u32> = OnceLock::new();

/// Store the real (unprivileged) UID of the process owner.
///
/// Must be called once from `main()` before any flash operation.
/// On non-Unix platforms this is a no-op.
pub fn set_real_uid(uid: u32) {
    let _ = REAL_UID.set(uid);
}

/// Retrieve the stored real UID, falling back to the current effective UID.
#[cfg(unix)]
fn real_uid() -> nix::unistd::Uid {
    let raw = REAL_UID
        .get()
        .copied()
        .unwrap_or_else(|| nix::unistd::getuid().as_raw());
    nix::unistd::Uid::from_raw(raw)
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A stage in the five-step flash pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashStage {
    /// Initial state before the pipeline starts.
    Starting,
    /// All partitions of the target device are being lazily unmounted.
    Unmounting,
    /// The image is being written to the block device in 4 MiB chunks.
    Writing,
    /// Kernel write-back caches are being flushed (`fsync` / `sync`).
    Syncing,
    /// The kernel is asked to re-read the partition table (`BLKRRPART`).
    Rereading,
    /// SHA-256 of the source image is compared against a read-back of the device.
    Verifying,
    /// The entire pipeline completed successfully.
    Done,
    /// The pipeline terminated with an error.
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
            FlashStage::Failed(m) => write!(f, "Failed: {m}"),
        }
    }
}

/// A typed event emitted by the flash pipeline.
///
/// Sent over [`std::sync::mpsc`] to the async Iced subscription — no
/// serialisation, no text parsing.
#[derive(Debug, Clone)]
pub enum FlashEvent {
    /// A pipeline stage transition.
    Stage(FlashStage),
    /// Write-progress update.
    Progress {
        bytes_written: u64,
        total_bytes: u64,
        speed_mb_s: f32,
    },
    /// Informational log message (not an error).
    Log(String),
    /// The pipeline finished successfully.
    Done,
    /// The pipeline failed; the string is a human-readable error.
    Error(String),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the full flash pipeline in the **calling thread**.
///
/// This function is blocking and must be called from a dedicated
/// `std::thread::spawn` thread, not from an async executor.
///
/// # Arguments
///
/// * `image_path`  – path to the source image file
/// * `device_path` – path to the target block device (e.g. `/dev/sdb`)
/// * `tx`          – channel to send [`FlashEvent`] progress updates
/// * `cancel`      – set to `true` to abort the pipeline between blocks
pub fn run_pipeline(
    image_path: &str,
    device_path: &str,
    tx: mpsc::Sender<FlashEvent>,
    cancel: Arc<AtomicBool>,
) {
    if let Err(e) = flash_pipeline(image_path, device_path, &tx, cancel) {
        let _ = tx.send(FlashEvent::Error(e));
    }
}

// ---------------------------------------------------------------------------
// Top-level pipeline
// ---------------------------------------------------------------------------

fn send(tx: &mpsc::Sender<FlashEvent>, event: FlashEvent) {
    // If the receiver is gone the GUI has been closed — ignore silently.
    let _ = tx.send(event);
}

fn flash_pipeline(
    image_path: &str,
    device_path: &str,
    tx: &mpsc::Sender<FlashEvent>,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    // ── Validate inputs ──────────────────────────────────────────────────────
    if !Path::new(image_path).is_file() {
        return Err(format!("Image file not found: {image_path}"));
    }

    if !Path::new(device_path).exists() {
        return Err(format!("Target device not found: {device_path}"));
    }

    // Guard against partition nodes (e.g. /dev/sdb1 instead of /dev/sdb).
    #[cfg(target_os = "linux")]
    reject_partition_node(device_path)?;

    let image_size = std::fs::metadata(image_path)
        .map_err(|e| format!("Cannot stat image: {e}"))?
        .len();

    if image_size == 0 {
        return Err("Image file is empty".to_string());
    }

    // ── Step 1: Unmount ──────────────────────────────────────────────────────
    send(tx, FlashEvent::Stage(FlashStage::Unmounting));
    unmount_device(device_path, tx);

    // ── Step 2: Write ────────────────────────────────────────────────────────
    send(tx, FlashEvent::Stage(FlashStage::Writing));
    send(
        tx,
        FlashEvent::Log(format!(
            "Writing {image_size} bytes from {image_path} → {device_path}"
        )),
    );
    write_image(image_path, device_path, image_size, tx, &cancel)?;

    // ── Step 3: Sync ─────────────────────────────────────────────────────────
    send(tx, FlashEvent::Stage(FlashStage::Syncing));
    sync_device(device_path, tx);

    // ── Step 4: Re-read partition table ──────────────────────────────────────
    send(tx, FlashEvent::Stage(FlashStage::Rereading));
    reread_partition_table(device_path, tx);

    // ── Step 5: Verify ───────────────────────────────────────────────────────
    send(tx, FlashEvent::Stage(FlashStage::Verifying));
    verify(image_path, device_path, image_size, tx)?;

    // ── Done ─────────────────────────────────────────────────────────────────
    send(tx, FlashEvent::Done);
    Ok(())
}

// ---------------------------------------------------------------------------
// Partition-node guard (Linux only)
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn reject_partition_node(device_path: &str) -> Result<(), String> {
    let dev_name = Path::new(device_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let is_partition = {
        let bytes = dev_name.as_bytes();
        !bytes.is_empty() && bytes[bytes.len() - 1].is_ascii_digit() && {
            let stem = dev_name.trim_end_matches(|c: char| c.is_ascii_digit());
            stem.ends_with('p')
                || (!stem.is_empty()
                    && !stem.ends_with(|c: char| c.is_ascii_digit())
                    && stem.chars().any(|c| c.is_ascii_alphabetic()))
        }
    };

    if is_partition {
        let whole = dev_name.trim_end_matches(|c: char| c.is_ascii_digit() || c == 'p');
        return Err(format!(
            "Refusing to write to partition node '{device_path}'. \
             Select the whole-disk device (e.g. /dev/{whole}) instead."
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Privilege helpers
// ---------------------------------------------------------------------------

/// Open `device_path` for raw writing, temporarily escalating to root if the
/// binary is setuid-root, then immediately dropping back to the real UID.
fn open_device_for_writing(device_path: &str) -> Result<std::fs::File, String> {
    #[cfg(unix)]
    {
        use nix::unistd::seteuid;

        // Attempt to escalate to root.
        //
        // This only succeeds when the binary carries the setuid-root bit
        // (`chmod u+s`).  If escalation fails we still try to open the file —
        // it may be a regular writable file (e.g. during tests) or the user
        // may already have write permission on the device.
        let escalated = seteuid(nix::unistd::Uid::from_raw(0)).is_ok();

        let result = std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| {
                let raw = e.raw_os_error().unwrap_or(0);
                if raw == libc::EACCES || raw == libc::EPERM {
                    if escalated {
                        format!(
                            "Permission denied opening '{device_path}'.\n\
                             Even with setuid-root the device refused access — \
                             check that the device exists and is not in use."
                        )
                    } else {
                        format!(
                            "Permission denied opening '{device_path}'.\n\
                             The binary is not installed setuid-root.\n\
                             Install with:\n  \
                             sudo chown root:root /usr/bin/flashkraft\n  \
                             sudo chmod u+s      /usr/bin/flashkraft"
                        )
                    }
                } else if raw == libc::EBUSY {
                    format!(
                        "Device '{device_path}' is busy. \
                         Ensure all partitions are unmounted before flashing."
                    )
                } else {
                    format!("Cannot open device '{device_path}' for writing: {e}")
                }
            });

        // Drop back to the real (unprivileged) user immediately.
        if escalated {
            let _ = seteuid(real_uid());
        }

        result
    }

    #[cfg(not(unix))]
    {
        std::fs::OpenOptions::new()
            .write(true)
            .open(device_path)
            .map_err(|e| format!("Cannot open device '{device_path}' for writing: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Step 1 – Unmount
// ---------------------------------------------------------------------------

fn unmount_device(device_path: &str, tx: &mpsc::Sender<FlashEvent>) {
    let device_name = Path::new(device_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let partitions = find_mounted_partitions(&device_name, device_path);

    if partitions.is_empty() {
        send(tx, FlashEvent::Log("No mounted partitions found".into()));
    } else {
        for partition in &partitions {
            send(tx, FlashEvent::Log(format!("Unmounting {partition}")));
            do_unmount(partition, tx);
        }
    }
}

fn find_mounted_partitions(device_name: &str, device_path: &str) -> Vec<String> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .or_else(|_| std::fs::read_to_string("/proc/self/mounts"))
        .unwrap_or_default();

    let mut partitions = Vec::new();

    for line in mounts.lines() {
        let dev = match line.split_whitespace().next() {
            Some(d) => d,
            None => continue,
        };
        if dev == device_path || is_partition_of(dev, device_name) {
            partitions.push(dev.to_string());
        }
    }

    partitions
}

fn is_partition_of(dev: &str, device_name: &str) -> bool {
    // `dev` may be a full path like "/dev/sda1"; compare only the basename.
    let dev_base = Path::new(dev)
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    if !dev_base.starts_with(device_name) {
        return false;
    }
    let suffix = &dev_base[device_name.len()..];
    if suffix.is_empty() {
        return false;
    }
    let first = suffix.chars().next().unwrap();
    first.is_ascii_digit() || (first == 'p' && suffix.len() > 1)
}

fn do_unmount(partition: &str, tx: &mpsc::Sender<FlashEvent>) {
    #[cfg(target_os = "linux")]
    {
        use nix::unistd::seteuid;
        use std::ffi::CString;

        // Need root to unmount.
        let _ = seteuid(nix::unistd::Uid::from_raw(0));

        if let Ok(c_path) = CString::new(partition) {
            let ret = unsafe { libc::umount2(c_path.as_ptr(), libc::MNT_DETACH) };
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                send(
                    tx,
                    FlashEvent::Log(format!("Warning — could not unmount {partition}: {err}")),
                );
            }
        }

        let _ = seteuid(real_uid());
    }

    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("diskutil")
            .args(["unmount", partition])
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                send(
                    tx,
                    FlashEvent::Log(format!("Warning — diskutil unmount {partition} failed")),
                );
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        send(
            tx,
            FlashEvent::Log(format!(
                "Unmounting not supported on this OS — skipping {partition}"
            )),
        );
    }
}

// ---------------------------------------------------------------------------
// Step 2 – Write image
// ---------------------------------------------------------------------------

fn write_image(
    image_path: &str,
    device_path: &str,
    image_size: u64,
    tx: &mpsc::Sender<FlashEvent>,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    let image_file =
        std::fs::File::open(image_path).map_err(|e| format!("Cannot open image: {e}"))?;

    let device_file = open_device_for_writing(device_path)?;

    let mut reader = io::BufReader::with_capacity(BLOCK_SIZE, image_file);
    let mut writer = io::BufWriter::with_capacity(BLOCK_SIZE, device_file);
    let mut buf = vec![0u8; BLOCK_SIZE];

    let mut bytes_written: u64 = 0;
    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
        // Honour cancellation requests between blocks.
        if cancel.load(Ordering::SeqCst) {
            return Err("Flash operation cancelled by user".to_string());
        }

        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Read error on image: {e}"))?;

        if n == 0 {
            break; // EOF
        }

        writer
            .write_all(&buf[..n])
            .map_err(|e| format!("Write error on device: {e}"))?;

        bytes_written += n as u64;

        let now = Instant::now();
        if now.duration_since(last_report) >= PROGRESS_INTERVAL || bytes_written >= image_size {
            let elapsed_s = now.duration_since(start).as_secs_f32();
            let speed_mb_s = if elapsed_s > 0.001 {
                (bytes_written as f32 / (1024.0 * 1024.0)) / elapsed_s
            } else {
                0.0
            };

            send(
                tx,
                FlashEvent::Progress {
                    bytes_written,
                    total_bytes: image_size,
                    speed_mb_s,
                },
            );
            last_report = now;
        }
    }

    // Flush BufWriter → kernel page cache.
    writer
        .flush()
        .map_err(|e| format!("Buffer flush error: {e}"))?;

    // Retrieve the underlying File for fsync.
    #[cfg_attr(not(unix), allow(unused_variables))]
    let device_file = writer
        .into_inner()
        .map_err(|e| format!("BufWriter error: {e}"))?;

    // fsync: push all dirty pages to the physical medium.
    // Treated as a hard error — a failed fsync means we cannot trust the
    // data reached the device.
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = device_file.as_raw_fd();
        let ret = unsafe { libc::fsync(fd) };
        if ret != 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!(
                "fsync failed on '{device_path}': {err} — \
                 data may not have been fully written to the device"
            ));
        }
    }

    // Emit a final progress event at 100 %.
    let elapsed_s = start.elapsed().as_secs_f32();
    let speed_mb_s = if elapsed_s > 0.001 {
        (bytes_written as f32 / (1024.0 * 1024.0)) / elapsed_s
    } else {
        0.0
    };
    send(
        tx,
        FlashEvent::Progress {
            bytes_written,
            total_bytes: image_size,
            speed_mb_s,
        },
    );

    send(tx, FlashEvent::Log("Image write complete".into()));
    Ok(())
}

// ---------------------------------------------------------------------------
// Step 3 – Sync
// ---------------------------------------------------------------------------

fn sync_device(device_path: &str, tx: &mpsc::Sender<FlashEvent>) {
    #[cfg(unix)]
    if let Ok(f) = std::fs::OpenOptions::new().write(true).open(device_path) {
        use std::os::unix::io::AsRawFd;
        let fd = f.as_raw_fd();
        #[cfg(target_os = "linux")]
        unsafe {
            libc::fdatasync(fd);
        }
        #[cfg(not(target_os = "linux"))]
        unsafe {
            libc::fsync(fd);
        }
        drop(f);
    }

    #[cfg(target_os = "linux")]
    unsafe {
        libc::sync();
    }

    send(
        tx,
        FlashEvent::Log("Kernel write-back caches flushed".into()),
    );
}

// ---------------------------------------------------------------------------
// Step 4 – Re-read partition table
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn reread_partition_table(device_path: &str, tx: &mpsc::Sender<FlashEvent>) {
    use nix::ioctl_none;
    use std::os::unix::io::AsRawFd;

    ioctl_none!(blkrrpart, 0x12, 95);

    // Brief pause so any pending I/O completes before we poke the kernel.
    std::thread::sleep(Duration::from_millis(500));

    match std::fs::OpenOptions::new().write(true).open(device_path) {
        Ok(f) => {
            let result = unsafe { blkrrpart(f.as_raw_fd()) };
            match result {
                Ok(_) => send(
                    tx,
                    FlashEvent::Log("Kernel partition table refreshed".into()),
                ),
                Err(e) => send(
                    tx,
                    FlashEvent::Log(format!(
                        "Warning — BLKRRPART ioctl failed \
                         (device may not be partitioned): {e}"
                    )),
                ),
            }
        }
        Err(e) => send(
            tx,
            FlashEvent::Log(format!(
                "Warning — could not open device for BLKRRPART: {e}"
            )),
        ),
    }
}

#[cfg(target_os = "macos")]
fn reread_partition_table(device_path: &str, tx: &mpsc::Sender<FlashEvent>) {
    let _ = std::process::Command::new("diskutil")
        .args(["rereadPartitionTable", device_path])
        .output();
    send(
        tx,
        FlashEvent::Log("Partition table refresh requested (macOS)".into()),
    );
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn reread_partition_table(_device_path: &str, tx: &mpsc::Sender<FlashEvent>) {
    send(
        tx,
        FlashEvent::Log("Partition table refresh not implemented on this platform".into()),
    );
}

// ---------------------------------------------------------------------------
// Step 5 – Verify
// ---------------------------------------------------------------------------

fn verify(
    image_path: &str,
    device_path: &str,
    image_size: u64,
    tx: &mpsc::Sender<FlashEvent>,
) -> Result<(), String> {
    send(
        tx,
        FlashEvent::Log("Computing SHA-256 of source image".into()),
    );
    let image_hash = sha256_first_n_bytes(image_path, image_size)?;

    send(
        tx,
        FlashEvent::Log(format!(
            "Reading back {image_size} bytes from device for verification"
        )),
    );
    let device_hash = sha256_first_n_bytes(device_path, image_size)?;

    if image_hash != device_hash {
        return Err(format!(
            "Verification failed — data mismatch \
             (image={image_hash} device={device_hash})"
        ));
    }

    send(
        tx,
        FlashEvent::Log(format!("Verification passed ({image_hash})")),
    );
    Ok(())
}

fn sha256_first_n_bytes(path: &str, max_bytes: u64) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open {path} for hashing: {e}"))?;

    let mut hasher = Sha256::new();
    let mut reader = io::BufReader::with_capacity(BLOCK_SIZE, file);
    let mut buf = vec![0u8; BLOCK_SIZE];
    let mut remaining = max_bytes;

    while remaining > 0 {
        let to_read = (remaining as usize).min(buf.len());
        let n = reader
            .read(&mut buf[..to_read])
            .map_err(|e| format!("Read error while hashing {path}: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::mpsc;

    fn make_channel() -> (mpsc::Sender<FlashEvent>, mpsc::Receiver<FlashEvent>) {
        mpsc::channel()
    }

    fn drain(rx: &mpsc::Receiver<FlashEvent>) -> Vec<FlashEvent> {
        let mut events = Vec::new();
        while let Ok(e) = rx.try_recv() {
            events.push(e);
        }
        events
    }

    fn has_stage(events: &[FlashEvent], stage: &FlashStage) -> bool {
        events
            .iter()
            .any(|e| matches!(e, FlashEvent::Stage(s) if s == stage))
    }

    fn find_error(events: &[FlashEvent]) -> Option<&str> {
        events.iter().find_map(|e| {
            if let FlashEvent::Error(msg) = e {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    // ── set_real_uid ────────────────────────────────────────────────────────

    #[test]
    fn test_set_real_uid_stores_value() {
        // OnceLock only sets once; in tests the first call wins.
        // Just verify it doesn't panic.
        set_real_uid(1000);
    }

    // ── is_partition_of ─────────────────────────────────────────────────────

    #[test]
    fn test_is_partition_of_sda() {
        assert!(is_partition_of("/dev/sda1", "sda"));
        assert!(is_partition_of("/dev/sda2", "sda"));
        assert!(!is_partition_of("/dev/sdb1", "sda"));
        assert!(!is_partition_of("/dev/sda", "sda"));
    }

    #[test]
    fn test_is_partition_of_nvme() {
        assert!(is_partition_of("/dev/nvme0n1p1", "nvme0n1"));
        assert!(is_partition_of("/dev/nvme0n1p2", "nvme0n1"));
        assert!(!is_partition_of("/dev/nvme0n1", "nvme0n1"));
    }

    #[test]
    fn test_is_partition_of_mmcblk() {
        assert!(is_partition_of("/dev/mmcblk0p1", "mmcblk0"));
        assert!(!is_partition_of("/dev/mmcblk0", "mmcblk0"));
    }

    #[test]
    fn test_is_partition_of_no_false_prefix_match() {
        assert!(!is_partition_of("/dev/sda1", "sd"));
    }

    // ── reject_partition_node ───────────────────────────────────────────────

    #[test]
    #[cfg(target_os = "linux")]
    fn test_reject_partition_node_sda1() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_reject_img.bin");
        std::fs::write(&img, vec![0u8; 1024]).unwrap();

        let result = reject_partition_node("/dev/sda1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Refusing"));

        let _ = std::fs::remove_file(img);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_reject_partition_node_nvme() {
        let result = reject_partition_node("/dev/nvme0n1p1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Refusing"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_reject_partition_node_accepts_whole_disk() {
        // /dev/sdb does not exist in CI — the function only checks the name
        // pattern, not whether the path exists.
        let result = reject_partition_node("/dev/sdb");
        assert!(result.is_ok(), "whole-disk node should not be rejected");
    }

    // ── find_mounted_partitions ──────────────────────────────────────────────

    #[test]
    fn test_find_mounted_partitions_parses_proc_mounts_format() {
        // We cannot mock /proc/mounts in a unit test so we just verify the
        // function doesn't panic and returns a Vec.
        let result = find_mounted_partitions("sda", "/dev/sda");
        let _ = result; // any result is valid
    }

    // ── sha256_first_n_bytes ─────────────────────────────────────────────────

    #[test]
    fn test_sha256_full_file() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("fk_sha256_full.bin");
        let data: Vec<u8> = (0u8..=255u8).cycle().take(4096).collect();
        std::fs::write(&path, &data).unwrap();

        let result = sha256_first_n_bytes(path.to_str().unwrap(), data.len() as u64).unwrap();
        let expected = format!("{:x}", Sha256::digest(&data));
        assert_eq!(result, expected);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_sha256_partial() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("fk_sha256_partial.bin");
        let data: Vec<u8> = (0u8..=255u8).cycle().take(8192).collect();
        std::fs::write(&path, &data).unwrap();

        let n = 4096u64;
        let result = sha256_first_n_bytes(path.to_str().unwrap(), n).unwrap();
        let expected = format!("{:x}", Sha256::digest(&data[..n as usize]));
        assert_eq!(result, expected);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_sha256_nonexistent_returns_error() {
        let result = sha256_first_n_bytes("/nonexistent/path.bin", 1024);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot open"));
    }

    #[test]
    fn test_sha256_empty_read_is_hash_of_empty() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("fk_sha256_empty.bin");
        std::fs::write(&path, b"hello world extended data").unwrap();

        // max_bytes = 0 → nothing is read → hash of empty input
        let result = sha256_first_n_bytes(path.to_str().unwrap(), 0).unwrap();
        let expected = format!("{:x}", Sha256::digest(b""));
        assert_eq!(result, expected);

        let _ = std::fs::remove_file(path);
    }

    // ── write_image (via temp files) ─────────────────────────────────────────

    #[test]
    fn test_write_image_to_temp_file() {
        let dir = std::env::temp_dir();
        let img_path = dir.join("fk_write_img.bin");
        let dev_path = dir.join("fk_write_dev.bin");

        let image_size: u64 = 2 * 1024 * 1024; // 2 MiB
        {
            let mut f = std::fs::File::create(&img_path).unwrap();
            let block: Vec<u8> = (0u8..=255u8).cycle().take(BLOCK_SIZE).collect();
            let mut rem = image_size;
            while rem > 0 {
                let n = rem.min(BLOCK_SIZE as u64) as usize;
                f.write_all(&block[..n]).unwrap();
                rem -= n as u64;
            }
        }
        std::fs::File::create(&dev_path).unwrap();

        let (tx, rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));

        let result = write_image(
            img_path.to_str().unwrap(),
            dev_path.to_str().unwrap(),
            image_size,
            &tx,
            &cancel,
        );

        assert!(result.is_ok(), "write_image failed: {result:?}");

        let written = std::fs::read(&dev_path).unwrap();
        let original = std::fs::read(&img_path).unwrap();
        assert_eq!(written, original, "written data must match image exactly");

        let events = drain(&rx);
        let has_progress = events
            .iter()
            .any(|e| matches!(e, FlashEvent::Progress { .. }));
        assert!(has_progress, "must emit at least one Progress event");

        let _ = std::fs::remove_file(img_path);
        let _ = std::fs::remove_file(dev_path);
    }

    #[test]
    fn test_write_image_cancelled_mid_write() {
        let dir = std::env::temp_dir();
        let img_path = dir.join("fk_cancel_img.bin");
        let dev_path = dir.join("fk_cancel_dev.bin");

        // Large enough that we definitely hit the cancel check.
        let image_size: u64 = 8 * 1024 * 1024; // 8 MiB
        {
            let mut f = std::fs::File::create(&img_path).unwrap();
            let block = vec![0xAAu8; BLOCK_SIZE];
            let mut rem = image_size;
            while rem > 0 {
                let n = rem.min(BLOCK_SIZE as u64) as usize;
                f.write_all(&block[..n]).unwrap();
                rem -= n as u64;
            }
        }
        std::fs::File::create(&dev_path).unwrap();

        let (tx, _rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(true)); // pre-cancelled

        let result = write_image(
            img_path.to_str().unwrap(),
            dev_path.to_str().unwrap(),
            image_size,
            &tx,
            &cancel,
        );

        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("cancelled"),
            "error should mention cancellation"
        );

        let _ = std::fs::remove_file(img_path);
        let _ = std::fs::remove_file(dev_path);
    }

    #[test]
    fn test_write_image_missing_image_returns_error() {
        let dir = std::env::temp_dir();
        let dev_path = dir.join("fk_noimg_dev.bin");
        std::fs::File::create(&dev_path).unwrap();

        let (tx, _rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));

        let result = write_image(
            "/nonexistent/image.img",
            dev_path.to_str().unwrap(),
            1024,
            &tx,
            &cancel,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot open image"));

        let _ = std::fs::remove_file(dev_path);
    }

    // ── verify ───────────────────────────────────────────────────────────────

    #[test]
    fn test_verify_matching_files() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_verify_img.bin");
        let dev = dir.join("fk_verify_dev.bin");
        let data = vec![0xBBu8; 64 * 1024];
        std::fs::write(&img, &data).unwrap();
        std::fs::write(&dev, &data).unwrap();

        let (tx, _rx) = make_channel();
        let result = verify(
            img.to_str().unwrap(),
            dev.to_str().unwrap(),
            data.len() as u64,
            &tx,
        );
        assert!(result.is_ok());

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    #[test]
    fn test_verify_mismatch_returns_error() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_mismatch_img.bin");
        let dev = dir.join("fk_mismatch_dev.bin");
        std::fs::write(&img, vec![0x00u8; 64 * 1024]).unwrap();
        std::fs::write(&dev, vec![0xFFu8; 64 * 1024]).unwrap();

        let (tx, _rx) = make_channel();
        let result = verify(img.to_str().unwrap(), dev.to_str().unwrap(), 64 * 1024, &tx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Verification failed"));

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    #[test]
    fn test_verify_only_checks_image_size_bytes() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_trunc_img.bin");
        let dev = dir.join("fk_trunc_dev.bin");
        let image_data = vec![0xCCu8; 32 * 1024];
        let mut device_data = image_data.clone();
        device_data.extend_from_slice(&[0xDDu8; 32 * 1024]);
        std::fs::write(&img, &image_data).unwrap();
        std::fs::write(&dev, &device_data).unwrap();

        let (tx, _rx) = make_channel();
        let result = verify(
            img.to_str().unwrap(),
            dev.to_str().unwrap(),
            image_data.len() as u64,
            &tx,
        );
        assert!(
            result.is_ok(),
            "should pass when first N bytes match: {result:?}"
        );

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    // ── flash_pipeline validation ────────────────────────────────────────────

    #[test]
    fn test_pipeline_rejects_missing_image() {
        let (tx, rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));
        run_pipeline("/nonexistent/image.iso", "/dev/null", tx, cancel);
        let events = drain(&rx);
        let err = find_error(&events);
        assert!(err.is_some(), "must emit an Error event");
        assert!(err.unwrap().contains("Image file not found"), "err={err:?}");
    }

    #[test]
    fn test_pipeline_rejects_empty_image() {
        let dir = std::env::temp_dir();
        let empty = dir.join("fk_empty.img");
        std::fs::write(&empty, b"").unwrap();

        let (tx, rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));
        run_pipeline(empty.to_str().unwrap(), "/dev/null", tx, cancel);

        let events = drain(&rx);
        let err = find_error(&events);
        assert!(err.is_some());
        assert!(err.unwrap().contains("empty"), "err={err:?}");

        let _ = std::fs::remove_file(empty);
    }

    #[test]
    fn test_pipeline_rejects_missing_device() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_nodev_img.bin");
        std::fs::write(&img, vec![0u8; 1024]).unwrap();

        let (tx, rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));
        run_pipeline(img.to_str().unwrap(), "/nonexistent/device", tx, cancel);

        let events = drain(&rx);
        let err = find_error(&events);
        assert!(err.is_some());
        assert!(
            err.unwrap().contains("Target device not found"),
            "err={err:?}"
        );

        let _ = std::fs::remove_file(img);
    }

    /// End-to-end pipeline test using only temp files (no real hardware).
    #[test]
    fn test_pipeline_end_to_end_temp_files() {
        let dir = std::env::temp_dir();
        let img = dir.join("fk_e2e_img.bin");
        let dev = dir.join("fk_e2e_dev.bin");

        let image_data: Vec<u8> = (0u8..=255u8).cycle().take(1024 * 1024).collect();
        std::fs::write(&img, &image_data).unwrap();
        std::fs::File::create(&dev).unwrap();

        let (tx, rx) = make_channel();
        let cancel = Arc::new(AtomicBool::new(false));
        run_pipeline(img.to_str().unwrap(), dev.to_str().unwrap(), tx, cancel);

        let events = drain(&rx);

        // Must have seen at least one Progress event.
        let has_progress = events
            .iter()
            .any(|e| matches!(e, FlashEvent::Progress { .. }));
        assert!(has_progress, "must emit Progress events");

        // On temp files the pipeline either completes (Done) or fails after
        // the write/verify stage (e.g. BLKRRPART on a regular file).
        let has_done = events.iter().any(|e| matches!(e, FlashEvent::Done));
        let has_error = events.iter().any(|e| matches!(e, FlashEvent::Error(_)));
        assert!(
            has_done || has_error,
            "pipeline must end with Done or Error"
        );

        if has_done {
            let written = std::fs::read(&dev).unwrap();
            assert_eq!(written, image_data, "written data must match image");
        } else if let Some(err_msg) = find_error(&events) {
            // Error must NOT be from write or verify.
            assert!(
                !err_msg.contains("Cannot open")
                    && !err_msg.contains("Verification failed")
                    && !err_msg.contains("Write error"),
                "unexpected error: {err_msg}"
            );
        }

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    // ── FlashStage Display ───────────────────────────────────────────────────

    #[test]
    fn test_flash_stage_display() {
        assert!(FlashStage::Writing.to_string().contains("Writing"));
        assert!(FlashStage::Syncing.to_string().contains("Flushing"));
        assert!(FlashStage::Done.to_string().contains("complete"));
        assert!(FlashStage::Failed("oops".into())
            .to_string()
            .contains("oops"));
    }
}
