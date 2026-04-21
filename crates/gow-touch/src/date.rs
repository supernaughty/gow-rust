//! Date and timestamp parsers for `touch`.
//!
//! - `parse_touch_date` handles `-d STRING` (human date) via parse_datetime (Q1).
//! - `parse_touch_stamp` handles `-t STAMP` (strict `[[CC]YY]MMDDhhmm[.ss]`;
//!   currently accepts only the unambiguous 12-digit `YYYYMMDDhhmm[.ss]` form).
//!
//! Reference: RESEARCH.md Q1 (jiff 0.2 + parse_datetime 0.14 — the exact uutils
//! combination) and RESEARCH.md Pitfall 9 (timezone handling).

use filetime::FileTime;
use jiff::Zoned;
use thiserror::Error;

/// Errors raised by `touch` parsers and timestamp I/O.
#[derive(Debug, Error)]
#[allow(dead_code)] // `Io` variant is consumed by `lib.rs` in Task 2.
pub enum TouchError {
    #[error("invalid date format '{0}': {1}")]
    InvalidDate(String, String),
    #[error("invalid timestamp '{0}': {1}")]
    InvalidStamp(String, String),
    #[error("io error on '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Parse a `-d STRING` date via `parse_datetime`. `reference` is the "now" anchor
/// for relative expressions like "yesterday" or "1 hour ago".
pub fn parse_touch_date(_date_str: &str, _reference: Zoned) -> Result<FileTime, TouchError> {
    todo!("RED — GREEN lands next commit")
}

/// Parse a `-t STAMP` strict timestamp: `YYYYMMDDhhmm[.ss]`.
pub fn parse_touch_stamp(_stamp: &str) -> Result<FileTime, TouchError> {
    todo!("RED — GREEN lands next commit")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::{Timestamp, Zoned};

    fn fixed_ref_utc() -> Zoned {
        // 2020-01-02T00:00:00 UTC
        let ts: Timestamp = "2020-01-02T00:00:00Z".parse().unwrap();
        ts.in_tz("UTC").unwrap()
    }

    #[test]
    fn parse_iso_timestamp() {
        let ft = parse_touch_date("2020-01-01T00:00:00Z", Zoned::now()).unwrap();
        // 2020-01-01T00:00:00Z unix = 1_577_836_800
        assert_eq!(ft.unix_seconds(), 1_577_836_800);
    }

    #[test]
    fn parse_yesterday_relative() {
        let ft = parse_touch_date("yesterday", fixed_ref_utc()).unwrap();
        // yesterday relative to 2020-01-02T00:00Z is 2020-01-01T00:00Z.
        // Allow ±1 day tolerance due to implementation variations (some parsers
        // snap to midnight in the reference's local tz, others preserve time-of-day).
        let expected: i64 = 1_577_836_800;
        let actual = ft.unix_seconds();
        assert!(
            (actual - expected).abs() < 86_400 * 2,
            "expected ~2020-01-01T00Z ({expected}), got unix {actual}"
        );
    }

    #[test]
    fn parse_hour_ago_relative() {
        let ft = parse_touch_date("1 hour ago", fixed_ref_utc()).unwrap();
        // 2020-01-02T00:00 - 1h = 2020-01-01T23:00:00Z = 1_577_919_600
        let expected: i64 = 1_577_919_600;
        assert!(
            (ft.unix_seconds() - expected).abs() < 60,
            "expected ~2020-01-01T23:00Z ({expected}), got unix {}",
            ft.unix_seconds()
        );
    }

    #[test]
    fn parse_now_keyword() {
        let ft = parse_touch_date("now", Zoned::now()).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(
            (ft.unix_seconds() - now).abs() < 5,
            "'now' should parse to within 5s of system clock; got {} vs {}",
            ft.unix_seconds(),
            now
        );
    }

    #[test]
    fn parse_invalid_date_returns_err() {
        let err = parse_touch_date("xyzzy not a date", fixed_ref_utc()).unwrap_err();
        assert!(matches!(err, TouchError::InvalidDate(_, _)));
    }

    #[test]
    fn stamp_12_digits() {
        let ft = parse_touch_stamp("202001010000").unwrap();
        // Should be 2020-01-01 00:00:00 in local timezone; cannot do exact
        // comparison because local TZ varies by test runner — bound to the year bucket.
        assert!(ft.unix_seconds() > 1_577_000_000 && ft.unix_seconds() < 1_578_000_000);
    }

    #[test]
    fn stamp_with_seconds() {
        let a = parse_touch_stamp("202001010000").unwrap();
        let b = parse_touch_stamp("202001010000.30").unwrap();
        assert_eq!(b.unix_seconds() - a.unix_seconds(), 30);
    }

    #[test]
    fn stamp_wrong_length_errors() {
        // 10 digits — not the accepted 12-digit form.
        let err = parse_touch_stamp("2020010100").unwrap_err();
        assert!(matches!(err, TouchError::InvalidStamp(_, _)));
    }

    #[test]
    fn stamp_non_digits_errors() {
        let err = parse_touch_stamp("abcdefghijkl").unwrap_err();
        assert!(matches!(err, TouchError::InvalidStamp(_, _)));
    }

    #[test]
    fn stamp_bad_seconds_field_errors() {
        // Dot present but only 1 digit after.
        let err = parse_touch_stamp("202001010000.5").unwrap_err();
        assert!(matches!(err, TouchError::InvalidStamp(_, _)));
    }
}
