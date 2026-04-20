//! UTF-8 console initialization.
//!
//! Sets the Windows console input and output code pages to UTF-8 (65001)
//! so that non-ASCII text (CJK filenames, emoji, etc.) renders correctly
//! in both Windows Terminal and legacy ConHost.
//!
//! The `activeCodePage=UTF-8` manifest entry in build.rs handles the ANSI
//! process code page (affects fopen, CreateFileA). This module handles the
//! console-specific code pages at runtime.
//!
//! Covers: FOUND-02, WIN-01

/// Set Windows console input and output code pages to UTF-8 (65001).
///
/// On non-Windows targets this is a no-op that compiles away.
/// Safe to call multiple times (idempotent).
#[cfg(target_os = "windows")]
pub fn setup_console_utf8() {
    use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
    // SAFETY: These are simple API calls with no memory ownership transfer.
    // Both calls are safe to make from any thread at any time.
    // Return values are intentionally ignored — failure to set code page is
    // non-fatal (the console may already be UTF-8 or may not exist, e.g. in
    // a piped context). We still proceed rather than panic.
    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn setup_console_utf8() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_console_utf8_does_not_panic() {
        // Must not panic regardless of platform or console availability.
        setup_console_utf8();
    }

    #[test]
    fn test_setup_console_utf8_is_idempotent() {
        // Calling twice must not error or panic.
        setup_console_utf8();
        setup_console_utf8();
    }
}
