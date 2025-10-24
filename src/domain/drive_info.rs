//! Drive Information Domain Model
//!
//! This module contains the DriveInfo struct which represents
//! information about a storage drive in the system.

/// Information about a storage drive
#[derive(Debug, Clone)]
pub struct DriveInfo {
    /// Name of the drive
    pub name: String,
    /// Mount point of the drive
    pub mount_point: String,
    /// Size of the drive in gigabytes
    pub size_gb: f64,
    /// Raw device path (e.g., /dev/sde)
    pub device_path: String,
}

impl DriveInfo {
    /// Create a new DriveInfo instance
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the drive
    /// * `mount_point` - Where the drive is mounted
    /// * `size_gb` - Size in gigabytes
    /// * `device_path` - Raw device path (e.g., /dev/sde)
    pub fn new(name: String, mount_point: String, size_gb: f64, device_path: String) -> Self {
        Self {
            name,
            mount_point,
            size_gb,
            device_path,
        }
    }

    /// Get a display string for the drive
    ///
    /// # Returns
    ///
    /// A formatted string showing name, size, and mount point
    #[allow(dead_code)]
    pub fn display_string(&self) -> String {
        format!(
            "{} - {:.2} GB ({})",
            self.name, self.size_gb, self.mount_point
        )
    }
}

impl PartialEq for DriveInfo {
    fn eq(&self, other: &Self) -> bool {
        self.device_path == other.device_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_display_string() {
        let drive = DriveInfo::new(
            "USB Drive".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        assert_eq!(drive.display_string(), "USB Drive - 32.00 GB (/media/usb)");
    }

    #[test]
    fn test_drive_equality() {
        let drive1 = DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        let drive2 = DriveInfo::new(
            "USB".to_string(),
            "/media/usb".to_string(),
            32.0,
            "/dev/sdb".to_string(),
        );
        let drive3 = DriveInfo::new(
            "USB2".to_string(),
            "/media/usb2".to_string(),
            32.0,
            "/dev/sdc".to_string(),
        );

        assert_eq!(drive1, drive2);
        assert_ne!(drive1, drive3);
    }
}
