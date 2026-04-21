//! PATHEXT resolution with test-friendly override.
//!
//! Resolution order per D-18d:
//!   1. GOW_PATHEXT env var (testing override)
//!   2. PATHEXT env var (system default on Windows)
//!   3. Built-in fallback: `.COM;.EXE;.BAT;.CMD`
//!
//! Reference: RESEARCH.md Q6 lines 477-495.

use std::ffi::OsString;

const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

/// Parse a semicolon-separated PATHEXT string into a vector of non-empty extensions.
/// Empty entries are filtered out so `.EXE;;.BAT` becomes `[".EXE", ".BAT"]`.
pub fn parse_pathext_string(s: &str) -> Vec<OsString> {
    s.split(';')
        .filter(|e| !e.is_empty())
        .map(OsString::from)
        .collect()
}

/// Load the active PATHEXT list.
pub fn load_pathext() -> Vec<OsString> {
    let raw = std::env::var_os("GOW_PATHEXT")
        .or_else(|| std::env::var_os("PATHEXT"))
        .unwrap_or_else(|| OsString::from(DEFAULT_PATHEXT));
    // to_string_lossy is acceptable — PATHEXT entries are ASCII (.EXE, .BAT, etc.)
    parse_pathext_string(&raw.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_returns_empty_vec() {
        assert_eq!(parse_pathext_string(""), Vec::<OsString>::new());
    }

    #[test]
    fn parse_single_extension() {
        assert_eq!(parse_pathext_string(".EXE"), vec![OsString::from(".EXE")]);
    }

    #[test]
    fn parse_multiple_extensions() {
        assert_eq!(
            parse_pathext_string(".EXE;.BAT"),
            vec![OsString::from(".EXE"), OsString::from(".BAT")]
        );
    }

    #[test]
    fn parse_skips_empty_entries() {
        assert_eq!(
            parse_pathext_string(".EXE;;.BAT;"),
            vec![OsString::from(".EXE"), OsString::from(".BAT")]
        );
    }

    #[test]
    fn parse_only_separators_returns_empty() {
        assert_eq!(parse_pathext_string(";;"), Vec::<OsString>::new());
    }

    #[test]
    fn parse_default_pathext_shape() {
        // Integration smoke: the hardcoded fallback parses to 4 entries.
        assert_eq!(
            parse_pathext_string(DEFAULT_PATHEXT),
            vec![
                OsString::from(".COM"),
                OsString::from(".EXE"),
                OsString::from(".BAT"),
                OsString::from(".CMD"),
            ]
        );
    }

    // NOTE: We deliberately avoid testing `load_pathext()` directly because it reads
    // from process env which is racy across #[test] threads. Integration tests in
    // tests/integration.rs exercise the full env-driven path via subprocess spawn,
    // where each assert_cmd::Command gets an isolated env.
}
