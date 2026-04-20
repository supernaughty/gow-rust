//! Unified error types for gow-rust utilities.
//! Covers: FOUND-05, D-09

// Placeholder — full implementation in Plan 03

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn make_io_error() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "test error")
    }

    #[test]
    fn test_custom_exit_code_is_1() {
        assert_eq!(GowError::Custom("bad input".to_string()).exit_code(), 1);
    }

    #[test]
    fn test_io_error_exit_code_is_1() {
        let err = GowError::Io {
            path: "file.txt".to_string(),
            source: make_io_error(),
        };
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_custom_display_format() {
        let err = GowError::Custom("something went wrong".to_string());
        assert_eq!(format!("{err}"), "something went wrong");
    }

    #[test]
    fn test_io_display_format_includes_path() {
        let err = GowError::Io {
            path: "myfile.txt".to_string(),
            source: make_io_error(),
        };
        let msg = format!("{err}");
        assert!(
            msg.contains("myfile.txt"),
            "Error message must include the path, got: {msg}"
        );
        assert!(
            msg.starts_with("cannot open"),
            "Error message must start with 'cannot open', got: {msg}"
        );
    }

    #[test]
    fn test_io_err_helper() {
        let e = io_err("path/to/file", make_io_error());
        assert_eq!(e.exit_code(), 1);
        let msg = format!("{e}");
        assert!(msg.contains("path/to/file"));
    }

    #[test]
    fn test_permission_denied_display() {
        let err = GowError::PermissionDenied {
            path: "/etc/shadow".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("/etc/shadow"), "got: {msg}");
        assert!(msg.contains("Permission denied"), "got: {msg}");
    }
}
