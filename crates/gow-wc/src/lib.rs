//! `uu_wc`: GNU `wc` — Unicode-aware line/word/byte/char counter (TEXT-03, D-17).
//!
//! RED phase: Counts + count_bytes defined with placeholder body. Tests wired to
//! drive the GREEN-phase implementation.

use std::ffi::OsString;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub lines: u64,
    pub words: u64,
    pub bytes: u64,
    pub chars: u64,
}

pub fn count_bytes(_bytes: &[u8]) -> Counts {
    todo!("RED phase — implementation lands in GREEN commit")
}

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("wc: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_empty() {
        let c = count_bytes(b"");
        assert_eq!(c, Counts { lines: 0, words: 0, bytes: 0, chars: 0 });
    }

    #[test]
    fn count_single_word_newline() {
        let c = count_bytes(b"hello\n");
        assert_eq!(c, Counts { lines: 1, words: 1, bytes: 6, chars: 6 });
    }

    #[test]
    fn count_two_words() {
        let c = count_bytes(b"hello world\n");
        assert_eq!(c, Counts { lines: 1, words: 2, bytes: 12, chars: 12 });
    }

    #[test]
    fn count_two_lines() {
        let c = count_bytes(b"line1\nline2\n");
        assert_eq!(c, Counts { lines: 2, words: 2, bytes: 12, chars: 12 });
    }

    #[test]
    fn count_korean_single_word() {
        // "안녕\n" — 안 (3 bytes UTF-8) + 녕 (3 bytes) + \n = 7 bytes, 3 scalar values
        let c = count_bytes("안녕\n".as_bytes());
        assert_eq!(c.bytes, 7);
        assert_eq!(c.chars, 3);
        assert_eq!(c.lines, 1);
        assert_eq!(c.words, 1);
    }

    #[test]
    fn count_korean_with_space() {
        // "안녕 세상\n" — 4 Korean chars (12 bytes) + space (1) + newline (1) = 14 bytes,
        // 6 scalar values, 2 words
        let c = count_bytes("안녕 세상\n".as_bytes());
        assert_eq!(c.bytes, 14);
        assert_eq!(c.chars, 6);
        assert_eq!(c.words, 2);
        assert_eq!(c.lines, 1);
    }

    #[test]
    fn count_invalid_utf8_no_panic() {
        // 0xFF 0xFE are invalid UTF-8 starters. bstr::chars yields U+FFFD for each.
        let c = count_bytes(&[0xFF, 0xFE, b'\n']);
        assert_eq!(c.bytes, 3);
        assert_eq!(c.lines, 1);
        // chars: at least 1 (for newline); exact count for invalid sequences depends on
        // bstr's error recovery. Key requirement: NO PANIC.
        assert!(c.chars >= 1);
    }

    #[test]
    fn count_unterminated_last_line() {
        // Final byte is not newline → GNU wc: "hello" counts as 0 lines (but 1 word).
        let c = count_bytes(b"hello");
        assert_eq!(c.lines, 0);
        assert_eq!(c.words, 1);
        assert_eq!(c.bytes, 5);
    }
}
