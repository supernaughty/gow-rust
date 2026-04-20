//! Unified error types for gow-rust utilities.
//!
//! Design:
//! - `GowError` is used in library code (gow-core, gow-cat, etc.) for typed errors.
//! - Binary `main()` functions use `anyhow` for error propagation and print errors
//!   in GNU format: `{utility}: {message}` before calling `process::exit(error.exit_code())`.
//!
//! Covers: FOUND-05, D-09, D-10, D-11

use thiserror::Error;

/// Typed error enum for gow-rust utilities.
///
/// All utility library functions return `Result<T, GowError>`. The binary
/// entry point (main.rs) matches on this to determine the exit code and
/// format the error message.
#[derive(Debug, Error)]
pub enum GowError {
    /// I/O error associated with a specific file path.
    ///
    /// Message format follows GNU convention:
    /// `cannot open '{path}': {source}`
    #[error("cannot open '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Generic error with a custom message.
    ///
    /// Used when a more specific variant is not warranted.
    /// The message is displayed as-is.
    #[error("{0}")]
    Custom(String),

    /// Permission denied accessing a path.
    #[error("cannot access '{path}': Permission denied")]
    PermissionDenied { path: String },

    /// A path that was expected to exist does not.
    #[error("'{path}': No such file or directory")]
    NotFound { path: String },
}

impl GowError {
    /// The GNU exit code for this error.
    ///
    /// GNU tools use exit code 1 for all operational errors.
    /// Exit code 2 is reserved for "misuse of shell builtins" (bash convention)
    /// or serious configuration errors — not used by gow-rust utilities.
    pub fn exit_code(&self) -> i32 {
        1
    }
}

/// Helper to create a `GowError::Io` from an `std::io::Error` and a path.
///
/// Convenience function for the common pattern:
/// ```
/// use gow_core::error::io_err;
/// let result: Result<(), _> = std::fs::read("missing.txt")
///     .map(|_| ())
///     .map_err(|e| io_err("missing.txt", e));
/// ```
pub fn io_err(path: impl Into<String>, source: std::io::Error) -> GowError {
    GowError::Io {
        path: path.into(),
        source,
    }
}

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
