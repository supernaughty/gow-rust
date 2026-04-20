//! UTF-8 console initialization (SetConsoleOutputCP 65001 + SetConsoleCP 65001).
//! Covers: FOUND-02, WIN-01

#[cfg(target_os = "windows")]
pub fn setup_console_utf8() {
    // Placeholder — full implementation in Plan 02
}

#[cfg(not(target_os = "windows"))]
pub fn setup_console_utf8() {}
