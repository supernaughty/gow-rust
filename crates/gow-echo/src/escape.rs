//! GNU `echo -e` escape-sequence state machine.
//!
//! Parses `\n`, `\t`, `\r`, `\f`, `\v`, `\a`, `\b`, `\e`, `\\`, `\0NNN` (octal, up
//! to 3 digits), `\xHH` (hex, up to 2 digits), and `\c` (early break — see Control).
//! Unknown escapes (`\z`) are preserved verbatim, matching GNU's fallback.
//!
//! Reference: RESEARCH.md Q9 lines 754-880. Unit tests in this file.

use std::io::{self, Write};

/// Controls whether the caller should append a trailing newline after `write_escaped` returns.
/// `Break` means `\c` was encountered — caller MUST NOT append the newline.
#[derive(Debug, PartialEq, Eq)]
pub enum Control {
    Continue,
    Break,
}

/// Write `bytes` to `out`, interpreting backslash escapes per GNU `echo -e`.
///
/// Returns `Control::Break` if a `\c` sequence was encountered (caller must suppress
/// any trailing newline). Otherwise returns `Control::Continue`.
pub fn write_escaped<W: Write>(bytes: &[u8], out: &mut W) -> io::Result<Control> {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b != b'\\' {
            out.write_all(&[b])?;
            i += 1;
            continue;
        }
        // Starts escape.
        if i + 1 >= bytes.len() {
            // Trailing bare `\` — preserve literal.
            out.write_all(b"\\")?;
            return Ok(Control::Continue);
        }
        match bytes[i + 1] {
            b'\\' => {
                out.write_all(b"\\")?;
                i += 2;
            }
            b'a' => {
                out.write_all(&[0x07])?;
                i += 2;
            }
            b'b' => {
                out.write_all(&[0x08])?;
                i += 2;
            }
            b'c' => return Ok(Control::Break),
            b'e' => {
                out.write_all(&[0x1B])?;
                i += 2;
            }
            b'f' => {
                out.write_all(&[0x0C])?;
                i += 2;
            }
            b'n' => {
                out.write_all(&[0x0A])?;
                i += 2;
            }
            b'r' => {
                out.write_all(&[0x0D])?;
                i += 2;
            }
            b't' => {
                out.write_all(&[0x09])?;
                i += 2;
            }
            b'v' => {
                out.write_all(&[0x0B])?;
                i += 2;
            }
            b'0' => {
                let (byte, consumed) = parse_octal(&bytes[i + 2..]);
                out.write_all(&[byte])?;
                i += 2 + consumed;
            }
            b'x' => {
                let (byte, consumed) = parse_hex(&bytes[i + 2..]);
                if consumed == 0 {
                    // No hex digit after \x — preserve literal "\x" and continue
                    // at the char after 'x'.
                    out.write_all(b"\\x")?;
                    i += 2;
                } else {
                    out.write_all(&[byte])?;
                    i += 2 + consumed;
                }
            }
            other => {
                // Unknown escape → emit backslash + char verbatim (GNU behavior).
                out.write_all(&[b'\\', other])?;
                i += 2;
            }
        }
    }
    Ok(Control::Continue)
}

/// Parse up to 3 octal digits from the start of `rest`. Returns (byte, digits_consumed).
pub fn parse_octal(rest: &[u8]) -> (u8, usize) {
    let mut val: u8 = 0;
    let mut n = 0;
    while n < 3 && n < rest.len() && (b'0'..=b'7').contains(&rest[n]) {
        val = val.wrapping_mul(8).wrapping_add(rest[n] - b'0');
        n += 1;
    }
    (val, n)
}

/// Parse up to 2 hex digits from the start of `rest`. Returns (byte, digits_consumed).
pub fn parse_hex(rest: &[u8]) -> (u8, usize) {
    let mut val: u8 = 0;
    let mut n = 0;
    while n < 2 && n < rest.len() && rest[n].is_ascii_hexdigit() {
        let digit = match rest[n] {
            b'0'..=b'9' => rest[n] - b'0',
            b'a'..=b'f' => rest[n] - b'a' + 10,
            b'A'..=b'F' => rest[n] - b'A' + 10,
            _ => unreachable!(),
        };
        val = val.wrapping_mul(16).wrapping_add(digit);
        n += 1;
    }
    (val, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &[u8]) -> (Vec<u8>, Control) {
        let mut buf = Vec::new();
        let c = write_escaped(input, &mut buf).expect("write to Vec cannot fail");
        (buf, c)
    }

    #[test]
    fn plain_text_passes_through() {
        let (buf, c) = run(b"plain text");
        assert_eq!(buf, b"plain text");
        assert_eq!(c, Control::Continue);
    }

    #[test]
    fn tab_escape() {
        let (buf, _) = run(b"a\\tb");
        assert_eq!(buf, b"a\tb");
    }

    #[test]
    fn backslash_c_breaks() {
        let (buf, c) = run(b"a\\cb");
        assert_eq!(buf, b"a");
        assert_eq!(c, Control::Break);
    }

    #[test]
    fn octal_escape() {
        let (buf, _) = run(b"\\0101"); // octal 101 = 0x41 = 'A'
        assert_eq!(buf, b"A");
    }

    #[test]
    fn hex_escape() {
        let (buf, _) = run(b"\\x41");
        assert_eq!(buf, b"A");
    }

    #[test]
    fn unknown_escape_preserved() {
        let (buf, _) = run(b"\\z");
        assert_eq!(buf, b"\\z");
    }

    #[test]
    fn trailing_bare_backslash() {
        let (buf, _) = run(b"abc\\");
        assert_eq!(buf, b"abc\\");
    }

    #[test]
    fn hex_without_digits_preserved() {
        let (buf, _) = run(b"\\xZZ");
        // \x with no hex digit → preserve literal \x, then ZZ pass through.
        assert_eq!(buf, b"\\xZZ");
    }

    #[test]
    fn parse_octal_basic() {
        assert_eq!(parse_octal(b"101xxx"), (0o101, 3));
    }

    #[test]
    fn parse_octal_non_octal_digit() {
        assert_eq!(parse_octal(b"9aaa"), (0, 0));
    }

    #[test]
    fn parse_hex_basic() {
        assert_eq!(parse_hex(b"4Agarbage"), (0x4A, 2));
    }

    #[test]
    fn parse_hex_non_hex() {
        assert_eq!(parse_hex(b"G"), (0, 0));
    }

    #[test]
    fn parse_octal_stops_at_3_digits() {
        // 4th '0' NOT consumed.
        assert_eq!(parse_octal(b"1010"), (0o101, 3));
    }

    #[test]
    fn newline_and_escape_seq() {
        // \n = 0x0A, \e = 0x1B
        let (buf, _) = run(b"a\\nb\\ec");
        assert_eq!(buf, b"a\nb\x1Bc");
    }

    #[test]
    fn escape_eof_immediately_after_c_breaks() {
        let (buf, c) = run(b"ab\\c");
        assert_eq!(buf, b"ab");
        assert_eq!(c, Control::Break);
    }

    #[test]
    fn all_simple_escapes() {
        let (buf, _) = run(b"\\a\\b\\e\\f\\n\\r\\t\\v\\\\");
        assert_eq!(buf, b"\x07\x08\x1B\x0C\x0A\x0D\x09\x0B\\");
    }
}
