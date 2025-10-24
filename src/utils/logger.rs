//! Debug Logging Utilities
//!
//! Centralized logging macros and utilities for FlashKraft.
//! All debug logging is conditionally compiled and only active in debug builds.

/// Debug logging macro for general application messages
///
/// Only prints in debug builds. Automatically prefixes with [DEBUG].
///
/// # Example
/// ```
/// debug_log!("User selected image: {}", path);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] {}", format!($($arg)*));
    };
}

/// Debug logging macro for flash subscription messages
///
/// Only prints in debug builds. Automatically prefixes with [FLASH_DEBUG].
///
/// # Example
/// ```
/// flash_debug!("Progress: {:.1}%", progress * 100.0);
/// ```
#[macro_export]
macro_rules! flash_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[FLASH_DEBUG] {}", format!($($arg)*));
    };
}

/// Debug logging macro for status messages
///
/// Only prints in debug builds. Automatically prefixes with [STATUS].
///
/// # Example
/// ```
/// status_log!("Starting flash operation");
/// ```
#[macro_export]
macro_rules! status_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[STATUS] {}", format!($($arg)*));
    };
}

/// Conditional debug logging - only logs if condition is true
///
/// # Example
/// ```
/// debug_if!(verbose_mode, "Detailed info: {:?}", data);
/// ```
#[macro_export]
macro_rules! debug_if {
    ($condition:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        if $condition {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_debug_macros_compile() {
        // These should compile without errors
        debug_log!("Test message");
        flash_debug!("Flash test: {}", 42);
        status_log!("Status test");
        debug_if!(true, "Conditional test");
    }
}
