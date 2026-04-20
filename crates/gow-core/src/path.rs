//! Context-aware MSYS/Unix to Windows path conversion.
//! Covers: FOUND-06, D-06, D-07, D-08

// Placeholder — full implementation in Plan 03

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
