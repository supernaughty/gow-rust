//! ANSI/VT100 color support and TTY detection.
//!
//! On Windows, ANSI escape sequences require `ENABLE_VIRTUAL_TERMINAL_PROCESSING`
//! to be set on the console handle. This module enables that flag at startup
//! (called from `gow_core::init()`).
//!
//! Uses `termcolor` for all color output — it handles both ANSI mode and the
//! legacy Windows Console API fallback automatically.
//!
//! Covers: FOUND-04

use termcolor::{ColorChoice, StandardStream};

/// Enable VT100/ANSI processing on Windows stdout.
///
/// Without this, ANSI escape sequences appear as raw characters in
/// legacy cmd.exe and ConHost. This call is idempotent and safe.
///
/// On non-Windows targets this is a no-op that compiles away.
#[cfg(target_os = "windows")]
pub fn enable_vt_mode() {
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_OUTPUT_HANDLE,
        SetConsoleMode,
    };
    // SAFETY: Standard Win32 console mode API. No memory ownership.
    // Failure is silently ignored — if VT mode cannot be enabled (e.g. output
    // is not a console, or we're running in a context without a console),
    // termcolor falls back to no-color mode automatically.
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn enable_vt_mode() {}

/// Determine the appropriate `ColorChoice` from a `--color` argument and environment.
///
/// Priority (highest to lowest):
/// 1. `NO_COLOR` environment variable (https://no-color.org/) — forces Never
/// 2. `--color` argument value: "always", "never", "auto"
/// 3. Default: Auto (termcolor performs isatty check internally)
///
/// # Arguments
/// * `arg` — The value of the `--color` flag, if provided. Pass `None` if the
///   flag was not given.
pub fn color_choice(arg: Option<&str>) -> ColorChoice {
    // NO_COLOR takes absolute priority per the spec.
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorChoice::Never;
    }
    match arg {
        Some("always") => ColorChoice::Always,
        Some("never") => ColorChoice::Never,
        Some("auto") | None => ColorChoice::Auto,
        // Unknown values fall back to Auto (lenient — GNU grep does the same).
        Some(_) => ColorChoice::Auto,
    }
}

/// Create a color-aware stdout writer.
///
/// This is a thin wrapper around `termcolor::StandardStream::stdout()`.
/// Use this instead of `std::io::stdout()` wherever color output is needed.
pub fn stdout(choice: ColorChoice) -> StandardStream {
    StandardStream::stdout(choice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_vt_mode_does_not_panic() {
        enable_vt_mode();
    }

    #[test]
    fn test_color_choice_always() {
        assert_eq!(color_choice(Some("always")), ColorChoice::Always);
    }

    #[test]
    fn test_color_choice_never() {
        assert_eq!(color_choice(Some("never")), ColorChoice::Never);
    }

    #[test]
    fn test_color_choice_auto_explicit() {
        assert_eq!(color_choice(Some("auto")), ColorChoice::Auto);
    }

    #[test]
    fn test_color_choice_default_is_auto() {
        // Only valid when NO_COLOR is not set in the test environment.
        // Guard against CI environments that set NO_COLOR.
        if std::env::var_os("NO_COLOR").is_none() {
            assert_eq!(color_choice(None), ColorChoice::Auto);
        }
    }

    #[test]
    fn test_stdout_does_not_panic() {
        let _s = stdout(ColorChoice::Never);
    }
}
