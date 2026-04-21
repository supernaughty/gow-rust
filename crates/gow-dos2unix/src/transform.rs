//! Shared byte-level transforms for `dos2unix` (CONV-01) and `unix2dos` (CONV-02).
//!
//! Both utilities live under this crate's `transform` module:
//! - `dos2unix` calls [`crlf_to_lf`]
//! - `unix2dos` (in crate `gow-unix2dos`) imports and calls [`lf_to_crlf`]
//!
//! Both share [`is_binary`] for the NUL-byte heuristic (D-47 / RESEARCH Pattern M).

/// First-N-bytes scan window for binary detection.
pub const BIN_SCAN_LIMIT: usize = 32 * 1024;

/// Convert DOS line endings (`\r\n`) to UNIX line endings (`\n`) byte-wise.
///
/// Bare `\r` (not followed by `\n`) is preserved — matches GNU `dos2unix`
/// default behavior (CR-line files are rare; preserving them avoids
/// data corruption on Classic Mac text).
pub fn crlf_to_lf(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'\r' && i + 1 < input.len() && input[i + 1] == b'\n' {
            out.push(b'\n');
            i += 2;
        } else {
            out.push(input[i]);
            i += 1;
        }
    }
    out
}

/// Convert UNIX line endings (`\n`) to DOS line endings (`\r\n`) byte-wise.
///
/// Pre-existing `\r\n` pairs are preserved (not doubled). Isolated `\r`
/// characters are preserved as-is.
pub fn lf_to_crlf(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + input.len() / 40);
    let mut prev: u8 = 0;
    for &b in input {
        if b == b'\n' && prev != b'\r' {
            out.push(b'\r');
            out.push(b'\n');
        } else {
            out.push(b);
        }
        prev = b;
    }
    out
}

/// Heuristic binary detection: returns true if any byte in the first
/// [`BIN_SCAN_LIMIT`] bytes is `0x00` (NUL). Matches GNU `dos2unix --info`
/// behavior and is adequate for refusing to mangle `.exe`/`.zip`/etc.
pub fn is_binary(input: &[u8]) -> bool {
    input.iter().take(BIN_SCAN_LIMIT).any(|&b| b == 0x00)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_to_lf_empty() {
        assert_eq!(crlf_to_lf(b""), Vec::<u8>::new());
    }

    #[test]
    fn crlf_to_lf_pure_lf_unchanged() {
        let input = b"a\nb\nc\n";
        assert_eq!(crlf_to_lf(input), input.to_vec());
    }

    #[test]
    fn crlf_to_lf_pure_crlf() {
        assert_eq!(crlf_to_lf(b"a\r\nb\r\n"), b"a\nb\n".to_vec());
    }

    #[test]
    fn crlf_to_lf_mixed() {
        assert_eq!(crlf_to_lf(b"a\r\nb\nc\r\n"), b"a\nb\nc\n".to_vec());
    }

    #[test]
    fn crlf_to_lf_preserves_bare_cr() {
        // Classic Mac file — bare \r preserved
        assert_eq!(crlf_to_lf(b"a\rb\rc"), b"a\rb\rc".to_vec());
    }

    #[test]
    fn crlf_to_lf_trailing_cr_alone() {
        // Final \r with no following \n is preserved
        assert_eq!(crlf_to_lf(b"abc\r"), b"abc\r".to_vec());
    }

    #[test]
    fn lf_to_crlf_empty() {
        assert_eq!(lf_to_crlf(b""), Vec::<u8>::new());
    }

    #[test]
    fn lf_to_crlf_pure_lf() {
        assert_eq!(lf_to_crlf(b"a\nb\n"), b"a\r\nb\r\n".to_vec());
    }

    #[test]
    fn lf_to_crlf_preserves_existing_crlf() {
        // Already DOS-formatted — don't double
        assert_eq!(lf_to_crlf(b"a\r\nb\r\n"), b"a\r\nb\r\n".to_vec());
    }

    #[test]
    fn lf_to_crlf_preserves_bare_cr() {
        assert_eq!(lf_to_crlf(b"a\rb"), b"a\rb".to_vec());
    }

    #[test]
    fn round_trip_lf_to_crlf_and_back() {
        let original = b"alpha\nbeta\ngamma\n";
        let roundtrip = crlf_to_lf(&lf_to_crlf(original));
        assert_eq!(roundtrip, original.to_vec());
    }

    #[test]
    fn round_trip_crlf_to_lf_and_back() {
        let original = b"alpha\r\nbeta\r\ngamma\r\n";
        let roundtrip = lf_to_crlf(&crlf_to_lf(original));
        assert_eq!(roundtrip, original.to_vec());
    }

    #[test]
    fn round_trip_with_utf8_korean() {
        // UTF-8 bytes for "안녕\n세계\n"
        let original = "안녕\n세계\n".as_bytes().to_vec();
        let roundtrip = crlf_to_lf(&lf_to_crlf(&original));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn is_binary_empty_no() {
        assert!(!is_binary(b""));
    }

    #[test]
    fn is_binary_text_no() {
        assert!(!is_binary(b"hello\nworld"));
    }

    #[test]
    fn is_binary_nul_yes() {
        assert!(is_binary(b"hello\0world"));
    }

    #[test]
    fn is_binary_nul_at_start_yes() {
        assert!(is_binary(b"\0abc"));
    }

    #[test]
    fn is_binary_nul_beyond_scan_limit_no() {
        let mut data = vec![b'a'; BIN_SCAN_LIMIT + 10];
        data[BIN_SCAN_LIMIT + 5] = 0x00;
        // NUL outside scan window — should NOT be detected
        assert!(!is_binary(&data));
    }

    #[test]
    fn is_binary_nul_at_scan_boundary_yes() {
        let mut data = vec![b'a'; BIN_SCAN_LIMIT];
        data[BIN_SCAN_LIMIT - 1] = 0x00;
        assert!(is_binary(&data));
    }
}
