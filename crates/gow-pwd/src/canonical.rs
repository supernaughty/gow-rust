//! UNC-aware canonical-path simplification. `std::fs::canonicalize` on Windows
//! always returns paths prefixed with `\\?\`. GNU `pwd -P` expects the prefix
//! stripped — BUT naive strip breaks true UNC paths (`\\?\UNC\server\share`)
//! and NT device paths (`\\?\GLOBALROOT\…`).
//!
//! This function replicates `dunce::simplified` safety rule inline, avoiding
//! the dunce workspace dependency (we only need the one function). Reference:
//! RESEARCH.md Q8 lines 685-712.

use std::path::{Path, PathBuf};

/// Strip the `\\?\` extended-length prefix ONLY when it precedes a DOS drive-letter
/// path (e.g., `\\?\X:\…`). Preserves:
/// - True UNC paths (`\\?\UNC\server\share\…`)
/// - NT device paths (`\\?\GLOBALROOT\…`, `\\?\pipe\…`)
/// - Paths without the prefix
///
/// Minimum drive-letter path length with prefix: 7 chars (`\\?\X:\` — 4 prefix + X + : + \).
pub fn simplify_canonical(p: &Path) -> PathBuf {
    let s = match p.to_str() {
        Some(s) => s,
        None => return p.to_path_buf(), // Non-UTF-8 path — leave alone
    };
    let bytes = s.as_bytes();

    if bytes.len() >= 7
        && bytes.starts_with(br"\\?\")
        && bytes[4].is_ascii_alphabetic()
        && bytes[5] == b':'
        && bytes[6] == b'\\'
    {
        PathBuf::from(&s[4..])
    } else {
        p.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn strips_drive_letter_prefix_uppercase() {
        let p = Path::new(r"\\?\C:\Users\foo");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"C:\Users\foo"));
    }

    #[test]
    fn strips_drive_letter_prefix_lowercase() {
        let p = Path::new(r"\\?\c:\Users\foo");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"c:\Users\foo"));
    }

    #[test]
    fn strips_minimal_drive_letter_path() {
        let p = Path::new(r"\\?\D:\");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"D:\"));
    }

    #[test]
    fn preserves_unc_prefix() {
        // True UNC path — MUST NOT strip. Mitigates T-02-04-02.
        let p = Path::new(r"\\?\UNC\server\share\file");
        assert_eq!(
            simplify_canonical(p),
            PathBuf::from(r"\\?\UNC\server\share\file")
        );
    }

    #[test]
    fn preserves_nt_device_path() {
        let p = Path::new(r"\\?\GLOBALROOT\Device\Foo");
        assert_eq!(
            simplify_canonical(p),
            PathBuf::from(r"\\?\GLOBALROOT\Device\Foo")
        );
    }

    #[test]
    fn passthrough_no_prefix() {
        let p = Path::new(r"C:\Users\foo");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"C:\Users\foo"));
    }

    #[test]
    fn passthrough_relative_path() {
        let p = Path::new(r"relative\path");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"relative\path"));
    }

    #[test]
    fn preserves_too_short_prefix_only() {
        // Exactly `\\?\` with nothing after — malformed but must not crash.
        let p = Path::new(r"\\?\");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"\\?\"));
    }

    #[test]
    fn preserves_prefix_without_drive_letter() {
        // `\\?\1:\` — first char after prefix is not alphabetic; leave as-is.
        let p = Path::new(r"\\?\1:\file");
        assert_eq!(simplify_canonical(p), PathBuf::from(r"\\?\1:\file"));
    }
}
