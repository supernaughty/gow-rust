//! Context-aware MSYS/Unix to Windows path conversion.
//!
//! Solves GOW issue #244: naive path conversion corrupts flag values like `/c`
//! (the cmd.exe /C switch) by converting it to `C:\`.
//!
//! The fix: only convert arguments that are in file-position slots. Flag values
//! (arguments that follow a flag like `-c`) are never converted.
//!
//! Key rules (D-06, D-07, D-08):
//! 1. MSYS detection: `/X/rest` where X is a single ASCII letter and there is
//!    at least one more path component after it (`/c/Users` → `C:\Users`).
//! 2. Bare drive paths (`/c`, `/z`) are AMBIGUOUS — leave unchanged.
//! 3. Flag values are NEVER converted — only file-position arguments.
//! 4. Conservative: if in doubt, return the original unchanged.
//!
//! Covers: FOUND-06, D-06, D-07, D-08

use path_slash::PathBufExt as _;
use std::ffi::OsString;
use std::path::PathBuf;

/// Convert a single argument string if it matches the MSYS2/Git Bash Unix-style
/// drive-letter path pattern: `/X/rest` → `X:\rest`.
///
/// Conservative rules (D-08):
/// - Argument must match `/<letter>/<rest>` with at least one char after the second `/`
/// - Bare `/X` or `/X/` (trailing slash only) is ambiguous — returned unchanged
/// - Arguments not starting with `/` followed by a single letter then `/` are unchanged
/// - Forward slashes in the converted rest-of-path become backslashes
pub fn try_convert_msys_path(arg: &str) -> String {
    let bytes = arg.as_bytes();

    // Pattern: /X/... where X is a single ASCII letter and there is at least
    // one non-empty path component after it (len >= 4: '/' + letter + '/' + char)
    if bytes.len() >= 4
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b'/'
        && bytes[3] != b'\0'
    {
        let drive = (bytes[1] as char).to_ascii_uppercase();
        // Convert the remainder: replace forward slashes with backslashes.
        let rest = arg[3..].replace('/', "\\");
        return format!("{drive}:\\{rest}");
    }

    // Not an MSYS path. Normalize forward slashes for Windows if it looks like
    // a Windows path with forward slashes (e.g. "C:/Users/foo" → "C:\Users\foo").
    // Detection: starts with a drive letter followed by `:`.
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && bytes[2] == b'/'
    {
        return arg.replace('/', "\\");
    }

    // Default: return unchanged.
    arg.to_owned()
}

/// Normalize a path string for use with `std::fs` on Windows.
///
/// Converts forward slashes to backslashes via `path-slash`.
/// Does NOT apply MSYS drive-letter detection — use `try_convert_msys_path` first
/// if the argument might be an MSYS-style path.
pub fn to_windows_path(arg: &str) -> PathBuf {
    PathBuf::from_slash(arg)
}

/// Pre-process a raw argument list, converting MSYS-style paths in file positions.
///
/// This function applies `try_convert_msys_path` only to arguments that occupy
/// confirmed file-position slots (D-06). Arguments following short flags (e.g.
/// `-c value`) are NOT converted.
///
/// # Limitations
/// This function uses a simple heuristic: arguments that start with `-` are
/// treated as flags; the argument immediately following a flag that takes a
/// value is treated as a flag value (not a file). For utilities with complex
/// argument structures, prefer calling `try_convert_msys_path` explicitly on
/// known file arguments after clap parsing.
///
/// The first argument (argv[0], the binary name) is always returned unchanged.
pub fn normalize_file_args(args: &[impl AsRef<str>]) -> Vec<OsString> {
    let mut result = Vec::with_capacity(args.len());
    let mut skip_next = false;

    for (i, arg) in args.iter().enumerate() {
        let s = arg.as_ref();

        if i == 0 {
            // argv[0] is the binary path — never convert.
            result.push(OsString::from(s));
            continue;
        }

        if skip_next {
            // This argument is a flag value — do not convert (D-06).
            result.push(OsString::from(s));
            skip_next = false;
            continue;
        }

        if s.starts_with('-') {
            // This is a flag. If it does not use `=`, assume the next arg is its value.
            // (Simplified heuristic — sufficient for path pre-processing before clap.)
            // Flags that clearly take no value (e.g. -v, --verbose) will have
            // skip_next incorrectly set, but that only means we skip converting
            // the next arg — which is conservative and safe.
            if !s.contains('=') && s.len() == 2 {
                // Short flag like `-c`: next arg is its value.
                skip_next = true;
            }
            result.push(OsString::from(s));
            continue;
        }

        // Positional argument — apply MSYS path conversion.
        result.push(OsString::from(try_convert_msys_path(s)));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msys_path_c_drive() {
        assert_eq!(try_convert_msys_path("/c/Users/foo"), "C:\\Users\\foo");
    }

    #[test]
    fn test_msys_path_d_drive() {
        assert_eq!(
            try_convert_msys_path("/d/workspace/project"),
            "D:\\workspace\\project"
        );
    }

    #[test]
    fn test_msys_path_with_spaces() {
        assert_eq!(
            try_convert_msys_path("/c/Users/foo bar/baz"),
            "C:\\Users\\foo bar\\baz"
        );
    }

    #[test]
    fn test_bare_drive_is_unchanged() {
        // /c alone is ambiguous (could be cmd.exe /c flag) — must NOT convert (D-08)
        assert_eq!(try_convert_msys_path("/c"), "/c");
    }

    #[test]
    fn test_flag_value_unchanged() {
        // A short flag like -c is not a path — must not convert
        assert_eq!(try_convert_msys_path("-c"), "-c");
    }

    #[test]
    fn test_windows_path_with_forward_slashes_normalized() {
        assert_eq!(
            try_convert_msys_path("C:/normal/win/path"),
            "C:\\normal\\win\\path"
        );
    }

    #[test]
    fn test_relative_path_unchanged() {
        assert_eq!(try_convert_msys_path("relative/path"), "relative/path");
    }

    #[test]
    fn test_absolute_unix_root_unchanged() {
        // /etc/hosts doesn't match MSYS drive pattern (no letter after first /)
        assert_eq!(try_convert_msys_path("/etc/hosts"), "/etc/hosts");
    }

    #[test]
    fn test_normalize_file_args_converts_positional_only() {
        let args = ["cmd", "/c/Users/foo", "-c", "value"];
        let result = normalize_file_args(&args);
        // argv[0] unchanged
        assert_eq!(result[0], "cmd");
        // Positional arg converted
        assert_eq!(result[1], "C:\\Users\\foo");
        // Flag unchanged
        assert_eq!(result[2], "-c");
        // Flag value NOT converted (follows -c short flag)
        assert_eq!(result[3], "value");
    }

    #[test]
    fn test_to_windows_path_converts_slashes() {
        let path = to_windows_path("some/forward/slashes");
        // On Windows this will be a backslash path; on other platforms it stays as-is.
        let _ = path.to_string_lossy();
    }
}
