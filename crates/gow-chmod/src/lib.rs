//! `uu_chmod`: GNU `chmod` — change file permissions (FILE-10).
//!
//! RED-phase stub: signatures only; tests exercise the intended behavior and
//! MUST fail until the GREEN-phase implementation lands in the next commit.

use std::ffi::OsString;

/// What the parsed mode string asks us to do to a file's read-only bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyTarget {
    SetReadOnly,
    ClearReadOnly,
    NoOpKeepCurrent,
}

/// Parse a GNU chmod mode string to the desired read-only-bit action.
/// RED stub: always returns an error so every positive assertion fails.
pub fn parse_mode(_s: &str) -> Result<ReadOnlyTarget, String> {
    Err("not implemented".to_string())
}

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("chmod: not yet implemented (RED phase)");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_octal_644_clears_ro() {
        assert_eq!(parse_mode("644").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_0644_clears_ro() {
        assert_eq!(parse_mode("0644").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_444_sets_ro() {
        assert_eq!(parse_mode("444").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_400_sets_ro() {
        assert_eq!(parse_mode("400").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_600_clears_ro() {
        assert_eq!(parse_mode("600").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_000_sets_ro() {
        assert_eq!(parse_mode("000").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_777_clears_ro() {
        assert_eq!(parse_mode("777").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_invalid_digit() {
        assert!(parse_mode("999").is_err());
        assert!(parse_mode("abc").is_err());
    }

    #[test]
    fn parse_symbolic_plus_w_clears() {
        assert_eq!(parse_mode("+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("u+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("a+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_symbolic_minus_w_sets() {
        assert_eq!(parse_mode("-w").unwrap(), ReadOnlyTarget::SetReadOnly);
        assert_eq!(parse_mode("u-w").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_symbolic_equal_r_sets_ro() {
        assert_eq!(parse_mode("u=r").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_symbolic_equal_rw_clears() {
        assert_eq!(parse_mode("u=rw").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_symbolic_group_only_is_noop() {
        assert_eq!(parse_mode("g+w").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
        assert_eq!(parse_mode("o-w").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
    }

    #[test]
    fn parse_symbolic_x_bit_is_noop() {
        assert_eq!(parse_mode("+x").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
        assert_eq!(parse_mode("u+x").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
    }

    #[test]
    fn parse_symbolic_empty_clause_errors() {
        assert!(parse_mode("").is_err());
    }

    #[test]
    fn parse_symbolic_multi_clause_last_wins() {
        assert_eq!(parse_mode("u-w,u+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("u+w,u-w").unwrap(), ReadOnlyTarget::SetReadOnly);
    }
}
