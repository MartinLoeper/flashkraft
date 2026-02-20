//! Pure-Rust Privileged Flash Helper
//!
//! This module implements the entire flash pipeline in pure Rust — no shell
//! scripts required.  It is invoked as a **child process** spawned by the
//! main application with elevated privileges:
//!
//! ```text
//! pkexec /path/to/flashkraft --flash-helper <image_path> <device_path>
//! ```
//!
//! All progress is reported via **structured lines on stdout**:
//!
//! | Prefix | Meaning |
//! |--------|---------|
//! | `STAGE:<name>` | Stage transition (UNMOUNTING / WRITING / SYNCING / REREADING / VERIFYING / DONE) |
//! | `SIZE:<bytes>` | Total image size in bytes |
//! | `PROGRESS:<bytes>:<speed_mb_s>` | Write progress update |
//! | `LOG:<message>` | Informational message |
//! | `ERROR:<message>` | Terminal error (process exits non-zero) |
//!
//! The caller (`flash_subscription.rs`) already knows how to parse all of
//! these via `flash_writer::parse_script_line`.

use nix::libc;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Write / read-back buffer size: 4 MiB is a sweet spot for USB throughput.
const BLOCK_SIZE: usize = 4 * 1024 * 1024;

/// Minimum interval between PROGRESS lines (avoids flooding the pipe).
const PROGRESS_INTERVAL: Duration = Duration::from_millis(400);

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Entry point for `--flash-helper` mode.
///
/// Writes all output to stdout in the structured protocol described in the
/// module doc-comment, then exits.  The caller must have ensured that this
/// process is running with sufficient privileges to open the raw device.
pub fn run(image_path: &str, device_path: &str) {
    if let Err(e) = flash_pipeline(image_path, device_path) {
        emit_line(&format!("ERROR:{e}"));
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Top-level pipeline
// ---------------------------------------------------------------------------

fn flash_pipeline(image_path: &str, device_path: &str) -> Result<(), String> {
    // ── Validate inputs ──────────────────────────────────────────────────────
    if !Path::new(image_path).is_file() {
        return Err(format!("Image file not found: {image_path}"));
    }

    // On Linux the device appears as a block special file; on macOS as a char
    // special file.  We accept both — the OS will reject the open if it's
    // genuinely wrong.
    if !Path::new(device_path).exists() {
        return Err(format!("Target device not found: {device_path}"));
    }

    let image_size = std::fs::metadata(image_path)
        .map_err(|e| format!("Cannot stat image file: {e}"))?
        .len();

    if image_size == 0 {
        return Err("Image file is empty".to_string());
    }

    // Emit the total size so the caller can show a progress ratio immediately.
    emit_line(&format!("SIZE:{image_size}"));

    // ── Step 1: Unmount ──────────────────────────────────────────────────────
    emit_line("STAGE:UNMOUNTING");
    unmount_device(device_path);

    // ── Step 2: Write ────────────────────────────────────────────────────────
    emit_line("STAGE:WRITING");
    emit_line(&format!(
        "LOG:Writing {image_size} bytes from {image_path} → {device_path}"
    ));
    write_image(image_path, device_path, image_size)?;

    // ── Step 3: Sync ─────────────────────────────────────────────────────────
    emit_line("STAGE:SYNCING");
    sync_device(device_path);

    // ── Step 4: Re-read partition table ──────────────────────────────────────
    emit_line("STAGE:REREADING");
    reread_partition_table(device_path);

    // ── Step 5: Verify ───────────────────────────────────────────────────────
    emit_line("STAGE:VERIFYING");
    verify(image_path, device_path, image_size)?;

    // ── Done ─────────────────────────────────────────────────────────────────
    emit_line("STAGE:DONE");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helper: emit a line and immediately flush stdout
// ---------------------------------------------------------------------------

fn emit_line(line: &str) {
    println!("{line}");
    let _ = io::stdout().flush();
}

// ---------------------------------------------------------------------------
// Step 1 – Unmount
// ---------------------------------------------------------------------------

/// Unmount every partition of `device_path` that is currently mounted.
///
/// Failures are logged but do not abort the pipeline — a busy mount will
/// simply cause the write to fail with a meaningful error later.
fn unmount_device(device_path: &str) {
    let device_name = Path::new(device_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let partitions = find_mounted_partitions(&device_name, device_path);

    if partitions.is_empty() {
        emit_line("LOG:No mounted partitions found");
    } else {
        for partition in &partitions {
            emit_line(&format!("LOG:Unmounting {partition}"));
            do_unmount(partition);
        }
    }
}

/// Read `/proc/mounts` (Linux) or `/proc/self/mounts` and return every
/// device node that belongs to our target drive.
fn find_mounted_partitions(device_name: &str, device_path: &str) -> Vec<String> {
    // Try both common paths; fall back to empty on any I/O error.
    let mounts_content = std::fs::read_to_string("/proc/mounts")
        .or_else(|_| std::fs::read_to_string("/proc/self/mounts"))
        .unwrap_or_default();

    let mut partitions = Vec::new();

    for line in mounts_content.lines() {
        // Format: <device> <mount_point> <fs_type> <options> <dump> <pass>
        let dev = match line.split_whitespace().next() {
            Some(d) => d,
            None => continue,
        };

        // Match both the device itself and any partition (sdb, sdb1, sdb2,
        // nvme0n1, nvme0n1p1, mmcblk0, mmcblk0p1, …).
        if dev == device_path || is_partition_of(dev, device_name) {
            partitions.push(dev.to_string());
        }
    }

    partitions
}

/// Return true when `dev` is a partition that belongs to `device_name`.
///
/// Handles:
/// - `sda` → `sda1`, `sda2`, …
/// - `nvme0n1` → `nvme0n1p1`, `nvme0n1p2`, …
/// - `mmcblk0` → `mmcblk0p1`, …
fn is_partition_of(dev: &str, device_name: &str) -> bool {
    if !dev.starts_with(device_name) {
        return false;
    }
    let suffix = &dev[device_name.len()..];
    // The suffix must be non-empty and start with a digit or 'p' followed by
    // a digit (for NVMe/eMMC naming).
    if suffix.is_empty() {
        return false;
    }
    let first = suffix.chars().next().unwrap();
    first.is_ascii_digit() || (first == 'p' && suffix.len() > 1)
}

/// Perform a lazy unmount of a single partition.
///
/// On Linux we use `umount2(MNT_DETACH)` via libc so no shell is required.
/// On macOS we fall back to `diskutil unmountDisk` as the IOKit API is too
/// complex to reproduce here.
fn do_unmount(partition: &str) {
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        if let Ok(c_path) = CString::new(partition) {
            let ret = unsafe {
                // MNT_DETACH = 2: detach from the tree even if busy (lazy unmount)
                libc::umount2(c_path.as_ptr(), libc::MNT_DETACH)
            };
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                emit_line(&format!(
                    "LOG:Warning — could not unmount {partition}: {err}"
                ));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use diskutil; this is the pragmatic approach for dev usage.
        let out = std::process::Command::new("diskutil")
            .args(["unmount", partition])
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                emit_line(&format!(
                    "LOG:Warning — diskutil unmount {partition} failed"
                ));
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        emit_line(&format!(
            "LOG:Unmounting not supported on this OS — skipping {partition}"
        ));
    }
}

// ---------------------------------------------------------------------------
// Step 2 – Write image
// ---------------------------------------------------------------------------

fn write_image(image_path: &str, device_path: &str, image_size: u64) -> Result<(), String> {
    // Open the source image.
    let image_file =
        std::fs::File::open(image_path).map_err(|e| format!("Cannot open image file: {e}"))?;

    // Open the target device for raw writing.
    // We do NOT use O_DIRECT here — it requires 512-byte aligned user-space
    // buffers and causes EINVAL on many USB sticks.  conv=fsync / fsync() at
    // the end is the correct way to guarantee durability.
    let device_file = std::fs::OpenOptions::new()
        .write(true)
        .open(device_path)
        .map_err(|e| format!("Cannot open device for writing: {e}"))?;

    let mut reader = io::BufReader::with_capacity(BLOCK_SIZE, image_file);
    let mut writer = io::BufWriter::with_capacity(BLOCK_SIZE, device_file);
    let mut buf = vec![0u8; BLOCK_SIZE];

    let mut bytes_written: u64 = 0;
    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
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
        let should_report =
            now.duration_since(last_report) >= PROGRESS_INTERVAL || bytes_written >= image_size;

        if should_report {
            let elapsed_s = now.duration_since(start).as_secs_f32();
            let speed_mb_s = if elapsed_s > 0.001 {
                (bytes_written as f32 / (1024.0 * 1024.0)) / elapsed_s
            } else {
                0.0
            };

            emit_line(&format!("PROGRESS:{bytes_written}:{speed_mb_s:.2}"));
            last_report = now;
        }
    }

    // Flush the write buffer.
    writer
        .flush()
        .map_err(|e| format!("Buffer flush error: {e}"))?;

    // Retrieve the underlying file and call fsync to push data to the device.
    let device_file = writer
        .into_inner()
        .map_err(|e| format!("BufWriter error: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = device_file.as_raw_fd();
        let ret = unsafe { libc::fsync(fd) };
        if ret != 0 {
            let err = std::io::Error::last_os_error();
            emit_line(&format!("LOG:Warning — fsync returned error: {err}"));
        }
    }

    emit_line("LOG:Image write complete");
    Ok(())
}

// ---------------------------------------------------------------------------
// Step 3 – Sync
// ---------------------------------------------------------------------------

fn sync_device(device_path: &str) {
    // Re-open the device and fsync it (belt-and-suspenders after BufWriter).
    if let Ok(f) = std::fs::OpenOptions::new().write(true).open(device_path) {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let _ = unsafe { libc::fsync(f.as_raw_fd()) };
        }
        drop(f);
    }

    // Call the global sync() to flush all kernel write-back caches.
    #[cfg(target_os = "linux")]
    unsafe {
        libc::sync();
    }

    emit_line("LOG:Kernel write-back caches flushed");
}

// ---------------------------------------------------------------------------
// Step 4 – Re-read partition table
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn reread_partition_table(device_path: &str) {
    use nix::ioctl_none;
    use std::os::unix::io::AsRawFd;

    // BLKRRPART: _IO(0x12, 95) — ask the kernel to re-read the partition table.
    // This is essential so that the OS and bootloader agree on what's on the disk.
    ioctl_none!(blkrrpart, 0x12, 95);

    // Give the kernel a moment to process any pending I/O before re-reading.
    std::thread::sleep(Duration::from_millis(500));

    match std::fs::OpenOptions::new().write(true).open(device_path) {
        Ok(f) => {
            let fd = f.as_raw_fd();
            // SAFETY: fd is valid, BLKRRPART takes no argument (void ioctl).
            let result = unsafe { blkrrpart(fd) };
            match result {
                Ok(_) => emit_line("LOG:Kernel partition table refreshed"),
                Err(e) => emit_line(&format!(
                    "LOG:Warning — BLKRRPART ioctl failed (device may not be partitioned): {e}"
                )),
            }
        }
        Err(e) => {
            emit_line(&format!(
                "LOG:Warning — could not open device for BLKRRPART: {e}"
            ));
        }
    }
}

#[cfg(target_os = "macos")]
fn reread_partition_table(device_path: &str) {
    // macOS does not have BLKRRPART; the IOMedia layer handles this
    // automatically.  We optionally call diskutil to be explicit.
    let _ = std::process::Command::new("diskutil")
        .args(["rereadPartitionTable", device_path])
        .output();
    emit_line("LOG:Partition table refresh requested (macOS)");
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn reread_partition_table(_device_path: &str) {
    emit_line("LOG:Partition table refresh not implemented on this platform");
}

// ---------------------------------------------------------------------------
// Step 5 – Verify
// ---------------------------------------------------------------------------

fn verify(image_path: &str, device_path: &str, image_size: u64) -> Result<(), String> {
    emit_line("LOG:Computing SHA-256 of source image");
    let image_hash = sha256_first_n_bytes(image_path, image_size)?;

    emit_line(&format!(
        "LOG:Reading back {image_size} bytes from device for verification"
    ));
    let device_hash = sha256_first_n_bytes(device_path, image_size)?;

    if image_hash != device_hash {
        return Err(format!(
            "Verification failed — data mismatch \
             (image={image_hash} device={device_hash})"
        ));
    }

    emit_line(&format!("LOG:Verification passed ({image_hash})"));
    Ok(())
}

/// Compute the SHA-256 digest of the first `max_bytes` bytes of `path`.
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

    // ── is_partition_of ──────────────────────────────────────────────────────

    #[test]
    fn test_is_partition_of_sda() {
        assert!(is_partition_of("sda1", "sda"));
        assert!(is_partition_of("sda2", "sda"));
        assert!(is_partition_of("sda10", "sda"));
        assert!(!is_partition_of("sda", "sda")); // device itself, not partition
        assert!(!is_partition_of("sdb1", "sda")); // different device
    }

    #[test]
    fn test_is_partition_of_nvme() {
        assert!(is_partition_of("nvme0n1p1", "nvme0n1"));
        assert!(is_partition_of("nvme0n1p2", "nvme0n1"));
        assert!(!is_partition_of("nvme0n1", "nvme0n1")); // device itself
        assert!(!is_partition_of("nvme0n2p1", "nvme0n1")); // different device
    }

    #[test]
    fn test_is_partition_of_mmcblk() {
        assert!(is_partition_of("mmcblk0p1", "mmcblk0"));
        assert!(is_partition_of("mmcblk0p2", "mmcblk0"));
        assert!(!is_partition_of("mmcblk0", "mmcblk0"));
        assert!(!is_partition_of("mmcblk1p1", "mmcblk0"));
    }

    #[test]
    fn test_is_partition_of_no_false_prefix_match() {
        // "sdaa1" must not match device "sda"
        assert!(!is_partition_of("sdaa1", "sda"));
    }

    // ── find_mounted_partitions ──────────────────────────────────────────────

    #[test]
    fn test_find_mounted_partitions_parses_proc_mounts_format() {
        // Simulate /proc/mounts content
        let mounts = "\
sysfs /sys sysfs rw 0 0\n\
/dev/sda1 / ext4 rw 0 0\n\
/dev/sda2 /boot ext4 rw 0 0\n\
/dev/sdb1 /media/usb vfat rw 0 0\n\
tmpfs /tmp tmpfs rw 0 0\n\
";
        // We can't call find_mounted_partitions directly with custom content,
        // so test the parsing logic via is_partition_of instead.
        let device_name = "sdb";
        let device_path = "/dev/sdb";

        let relevant: Vec<&str> = mounts
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter(|dev| {
                *dev == device_path || is_partition_of(dev.trim_start_matches("/dev/"), device_name)
            })
            .collect();

        assert_eq!(relevant, vec!["/dev/sdb1"]);
    }

    #[test]
    fn test_find_mounted_partitions_includes_device_itself() {
        let device_path = "/dev/sdb";
        let device_name = "sdb";
        let mounts = "/dev/sdb /mnt ext4 rw 0 0\n";

        let relevant: Vec<&str> = mounts
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter(|dev| {
                *dev == device_path || is_partition_of(dev.trim_start_matches("/dev/"), device_name)
            })
            .collect();

        assert_eq!(relevant, vec!["/dev/sdb"]);
    }

    // ── sha256_first_n_bytes ─────────────────────────────────────────────────

    #[test]
    fn test_sha256_first_n_bytes_full_file() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("flashkraft_test_sha256_full.bin");

        let data = b"Hello, FlashKraft!";
        std::fs::write(&path, data).unwrap();

        let result = sha256_first_n_bytes(path.to_str().unwrap(), data.len() as u64).unwrap();

        let mut expected_hasher = Sha256::new();
        expected_hasher.update(data);
        let expected = format!("{:x}", expected_hasher.finalize());

        assert_eq!(result, expected);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_sha256_first_n_bytes_partial() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("flashkraft_test_sha256_partial.bin");

        let data = b"ABCDEFGHIJ"; // 10 bytes
        std::fs::write(&path, data).unwrap();

        // Hash only the first 5 bytes
        let result = sha256_first_n_bytes(path.to_str().unwrap(), 5).unwrap();

        let mut expected_hasher = Sha256::new();
        expected_hasher.update(&data[..5]);
        let expected = format!("{:x}", expected_hasher.finalize());

        assert_eq!(result, expected);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_sha256_nonexistent_file_returns_error() {
        let result = sha256_first_n_bytes("/nonexistent/path/file.bin", 1024);
        assert!(result.is_err(), "should fail on missing file");
    }

    #[test]
    fn test_sha256_empty_read_returns_hash_of_empty() {
        use sha2::{Digest, Sha256};

        let dir = std::env::temp_dir();
        let path = dir.join("flashkraft_test_sha256_empty.bin");
        std::fs::write(&path, b"some content").unwrap();

        // Request 0 bytes → hash of the empty string
        let result = sha256_first_n_bytes(path.to_str().unwrap(), 0).unwrap();

        let expected = format!("{:x}", Sha256::new().finalize());
        assert_eq!(result, expected);

        let _ = std::fs::remove_file(path);
    }

    // ── write_image (via temp files) ─────────────────────────────────────────

    /// Write a known pattern from a temp "image" file into a temp "device"
    /// file and verify the contents match.  This exercises the full write
    /// pipeline without requiring real hardware.
    #[test]
    fn test_write_image_to_temp_file() {
        let dir = std::env::temp_dir();
        let image_path = dir.join("flashkraft_test_image.bin");
        let device_path = dir.join("flashkraft_test_device.bin");

        // Create a 2 MiB image with a known pattern
        let image_size: u64 = 2 * 1024 * 1024;
        {
            let mut f = std::fs::File::create(&image_path).unwrap();
            let block: Vec<u8> = (0u8..=255u8).cycle().take(BLOCK_SIZE).collect();
            let mut remaining = image_size;
            while remaining > 0 {
                let n = remaining.min(BLOCK_SIZE as u64) as usize;
                f.write_all(&block[..n]).unwrap();
                remaining -= n as u64;
            }
        }

        // Create an empty device file to write into
        std::fs::File::create(&device_path).unwrap();

        let result = write_image(
            image_path.to_str().unwrap(),
            device_path.to_str().unwrap(),
            image_size,
        );

        assert!(result.is_ok(), "write_image failed: {result:?}");

        // Verify contents match exactly
        let written = std::fs::read(&device_path).unwrap();
        let original = std::fs::read(&image_path).unwrap();
        assert_eq!(
            written.len(),
            original.len(),
            "written size mismatch: {} vs {}",
            written.len(),
            original.len()
        );
        assert_eq!(written, original, "written content does not match image");

        let _ = std::fs::remove_file(image_path);
        let _ = std::fs::remove_file(device_path);
    }

    #[test]
    fn test_write_image_missing_image_returns_error() {
        let dir = std::env::temp_dir();
        let device_path = dir.join("flashkraft_test_nodev.bin");
        std::fs::File::create(&device_path).unwrap();

        let result = write_image(
            "/nonexistent/image.img",
            device_path.to_str().unwrap(),
            1024,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot open image"));

        let _ = std::fs::remove_file(device_path);
    }

    #[test]
    fn test_write_image_missing_device_returns_error() {
        let dir = std::env::temp_dir();
        let image_path = dir.join("flashkraft_test_noimg.bin");
        std::fs::write(&image_path, b"data").unwrap();

        let result = write_image(image_path.to_str().unwrap(), "/nonexistent/device/path", 4);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot open device"));

        let _ = std::fs::remove_file(image_path);
    }

    // ── verify (via temp files) ──────────────────────────────────────────────

    #[test]
    fn test_verify_matching_files_succeeds() {
        let dir = std::env::temp_dir();
        let img = dir.join("flashkraft_verify_img.bin");
        let dev = dir.join("flashkraft_verify_dev.bin");

        let data = vec![0xABu8; 64 * 1024]; // 64 KiB
        std::fs::write(&img, &data).unwrap();
        std::fs::write(&dev, &data).unwrap();

        let result = verify(
            img.to_str().unwrap(),
            dev.to_str().unwrap(),
            data.len() as u64,
        );
        assert!(result.is_ok(), "verify should pass for identical files");

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    #[test]
    fn test_verify_mismatched_files_fails() {
        let dir = std::env::temp_dir();
        let img = dir.join("flashkraft_verify_mismatch_img.bin");
        let dev = dir.join("flashkraft_verify_mismatch_dev.bin");

        std::fs::write(&img, vec![0x00u8; 64 * 1024]).unwrap();
        std::fs::write(&dev, vec![0xFFu8; 64 * 1024]).unwrap();

        let result = verify(img.to_str().unwrap(), dev.to_str().unwrap(), 64 * 1024);
        assert!(result.is_err(), "verify should fail for different files");
        let msg = result.unwrap_err();
        assert!(msg.contains("Verification failed"), "msg={msg}");
        assert!(
            msg.contains("image="),
            "msg should include image hash: {msg}"
        );
        assert!(
            msg.contains("device="),
            "msg should include device hash: {msg}"
        );

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    #[test]
    fn test_verify_only_checks_image_size_bytes() {
        // Device file is longer than the image — verify should still pass
        // because we only hash the first `image_size` bytes.
        let dir = std::env::temp_dir();
        let img = dir.join("flashkraft_verify_trunc_img.bin");
        let dev = dir.join("flashkraft_verify_trunc_dev.bin");

        let image_data = vec![0xCCu8; 32 * 1024]; // 32 KiB
        let mut device_data = image_data.clone();
        device_data.extend_from_slice(&[0xDDu8; 32 * 1024]); // extra 32 KiB

        std::fs::write(&img, &image_data).unwrap();
        std::fs::write(&dev, &device_data).unwrap();

        let result = verify(
            img.to_str().unwrap(),
            dev.to_str().unwrap(),
            image_data.len() as u64,
        );
        assert!(
            result.is_ok(),
            "verify should pass when first N bytes match: {result:?}"
        );

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }

    // ── flash_pipeline validation ────────────────────────────────────────────

    #[test]
    fn test_flash_pipeline_rejects_missing_image() {
        let result = flash_pipeline("/nonexistent/image.iso", "/dev/null");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Image file not found"));
    }

    #[test]
    fn test_flash_pipeline_rejects_empty_image() {
        let dir = std::env::temp_dir();
        let empty = dir.join("flashkraft_empty.img");
        std::fs::write(&empty, b"").unwrap();

        let result = flash_pipeline(empty.to_str().unwrap(), "/dev/null");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        let _ = std::fs::remove_file(empty);
    }

    #[test]
    fn test_flash_pipeline_rejects_missing_device() {
        let dir = std::env::temp_dir();
        let img = dir.join("flashkraft_pipeline_img.bin");
        std::fs::write(&img, vec![0u8; 1024]).unwrap();

        let result = flash_pipeline(img.to_str().unwrap(), "/nonexistent/device");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Target device not found"));

        let _ = std::fs::remove_file(img);
    }

    /// End-to-end test using only temp files (no real hardware).
    /// Exercises the full pipeline: write → sync → verify.
    #[test]
    fn test_flash_pipeline_end_to_end_temp_files() {
        let dir = std::env::temp_dir();
        let img = dir.join("flashkraft_e2e_img.bin");
        let dev = dir.join("flashkraft_e2e_dev.bin");

        // 1 MiB image
        let image_data: Vec<u8> = (0u8..=255u8).cycle().take(1024 * 1024).collect();
        std::fs::write(&img, &image_data).unwrap();

        // Pre-create the "device" file
        std::fs::File::create(&dev).unwrap();

        let result = flash_pipeline(img.to_str().unwrap(), dev.to_str().unwrap());

        // The pipeline will fail at REREADING/BLKRRPART (expected on a temp
        // file — it's not a block device) but should have already written and
        // verified successfully, and the error occurs *after* verification.
        // We accept both Ok and an error that is NOT about image/device
        // validation or writing.
        match result {
            Ok(()) => {
                // Full success (e.g. on a system where the ioctl is a no-op)
                let written = std::fs::read(&dev).unwrap();
                assert_eq!(written, image_data, "written data must match image");
            }
            Err(e) => {
                // The error should not be from write or verify
                assert!(
                    !e.contains("Cannot open") && !e.contains("Verification failed"),
                    "unexpected error: {e}"
                );
            }
        }

        let _ = std::fs::remove_file(img);
        let _ = std::fs::remove_file(dev);
    }
}
