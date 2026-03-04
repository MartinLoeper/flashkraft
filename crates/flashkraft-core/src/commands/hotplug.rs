//! USB Hotplug Detection
//!
//! Provides a cross-platform event stream that fires whenever a USB device is
//! connected or disconnected.  Callers receive a bare [`UsbHotplugEvent`] and
//! are expected to re-run [`crate::commands::load_drives_sync`] themselves —
//! hotplug is only a *trigger*, not a source of drive data.  All block-device
//! information (path, size, mount point) continues to come from the existing
//! sysfs / diskutil / wmic enumeration code.
//!
//! ## Why nusb for this and not inotify?
//!
//! `nusb::watch_devices()` works identically on Linux, macOS, and Windows
//! without any platform-specific code here.  On Linux it uses the kernel's
//! netlink USB socket (no `/dev/bus/usb` open required for *watching*).
//!
//! ## Permission notes
//!
//! `watch_devices()` **does not open any USB device node** — it only listens
//! for kernel announcements.  No udev rule, no `/dev/bus/usb` access, no
//! elevated privilege required.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use flashkraft_core::commands::hotplug::{watch_usb_events, UsbHotplugEvent};
//! use futures::StreamExt;
//!
//! let mut stream = watch_usb_events()?;
//! while let Some(event) = stream.next().await {
//!     match event {
//!         UsbHotplugEvent::Arrived  => println!("USB device connected"),
//!         UsbHotplugEvent::Left     => println!("USB device disconnected"),
//!     }
//!     // Re-enumerate block devices using the existing sysfs/diskutil path:
//!     let drives = flashkraft_core::commands::load_drives_sync();
//! }
//! ```

use nusb::hotplug::HotplugEvent;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A USB hotplug trigger emitted when any USB device connects or disconnects.
///
/// This is intentionally a plain enum with no device payload.  All drive
/// metadata is obtained by re-running the sysfs / diskutil / wmic enumeration
/// after receiving the event.  There is no point carrying `nusb::DeviceInfo`
/// here because it only knows about USB descriptors, not `/dev/sdX` paths or
/// mount points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsbHotplugEvent {
    /// A USB device was connected.
    Arrived,
    /// A USB device was disconnected.
    Left,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start watching for USB connect / disconnect events.
///
/// Returns a `Stream` that yields [`UsbHotplugEvent`] values.  The stream
/// runs for the lifetime of the returned object; drop it to stop watching.
///
/// # Errors
///
/// Returns an error if the OS refuses to create the watch (rare — typically
/// only happens if the system has no USB subsystem at all).
///
/// # Platform behaviour
///
/// | Platform | Mechanism |
/// |----------|-----------|
/// | Linux    | Kernel netlink USB socket via `nusb` (no `/dev/bus/usb` needed) |
/// | macOS    | IOKit run-loop notifications via `nusb` |
/// | Windows  | `RegisterDeviceNotification` via `nusb` |
pub fn watch_usb_events() -> Result<impl futures::Stream<Item = UsbHotplugEvent>, nusb::Error> {
    use futures::StreamExt;

    let watch = nusb::watch_devices()?;

    // Map the nusb HotplugEvent stream to our simpler UsbHotplugEvent.
    // We don't expose nusb::DeviceInfo to callers — all drive data comes
    // from the existing sysfs / diskutil / wmic enumeration code.
    let stream = watch.map(|event| match event {
        HotplugEvent::Connected(_) => UsbHotplugEvent::Arrived,
        HotplugEvent::Disconnected(_) => UsbHotplugEvent::Left,
    });

    Ok(stream)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// UsbHotplugEvent must be Clone, PartialEq, and Debug (used in assertions
    /// throughout the codebase and required by the Iced message derive).
    #[test]
    fn test_event_traits() {
        let a = UsbHotplugEvent::Arrived;
        let b = UsbHotplugEvent::Left;

        // Clone
        let a2 = a.clone();
        let b2 = b.clone();

        // PartialEq
        assert_eq!(a, a2);
        assert_eq!(b, b2);
        assert_ne!(a, b);

        // Debug (just verify it doesn't panic)
        assert!(format!("{a:?}").contains("Arrived"));
        assert!(format!("{b:?}").contains("Left"));
    }

    /// The two variants are distinct and exhaustively coverable.
    #[test]
    fn test_variant_exhaustiveness() {
        for event in [UsbHotplugEvent::Arrived, UsbHotplugEvent::Left] {
            let label = match event {
                UsbHotplugEvent::Arrived => "arrived",
                UsbHotplugEvent::Left => "left",
            };
            assert!(!label.is_empty());
        }
    }

    /// Calling `watch_usb_events()` on the test host should either succeed or
    /// return a clean error — it must never panic.
    ///
    /// We do not drive the stream here (that would require plugging in hardware),
    /// but constructing it exercises the OS code path for setting up the watch.
    #[test]
    fn test_watch_usb_events_does_not_panic() {
        // Either Ok(stream) or Err(nusb::Error) — both are acceptable.
        let result = watch_usb_events();
        // Log the outcome so CI logs are informative.
        match result {
            Ok(_) => println!("watch_usb_events: stream created successfully"),
            Err(ref e) => println!("watch_usb_events: OS returned error (acceptable): {e}"),
        }
        // The important thing is that we reach this line without panicking.
    }
}
