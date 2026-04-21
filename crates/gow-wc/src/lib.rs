//! `uu_wc`: GNU `wc` — Unicode-aware line/word/byte/char counter (TEXT-03, D-17).
//!
//! Unicode policy per D-17: always Unicode-aware, ignores `LANG` / `LC_CTYPE`.
//! Uses `bstr::chars()` which yields U+FFFD for invalid UTF-8 rather than
//! panicking (D-17b). `-L` (longest line display width) is deferred to v2 (D-17c).
//!
//! Flag semantics (D-17a):
//! - `-c` / `--bytes`  = raw byte count (`input.len()`)
//! - `-l` / `--lines`  = count of `b'\n'` in the raw bytes
//! - `-m` / `--chars`  = Unicode scalar values via `bstr::chars` (invalid → U+FFFD)
//! - `-w` / `--words`  = whitespace-delimited token count (`char::is_whitespace`)
//!
//! Default (no flag given): prints lines, words, bytes in that order (GNU wc).

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use bstr::ByteSlice;
use clap::{Arg, ArgAction, Command};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub lines: u64,
    pub words: u64,
    pub bytes: u64,
    pub chars: u64,
}

impl Counts {
    fn add(&mut self, other: &Counts) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
        self.chars += other.chars;
    }
}

/// Compute {lines, words, bytes, chars} for a byte slice.
///
/// Unicode policy: `bstr::chars()` yields one `char` per valid UTF-8 scalar and
/// one U+FFFD per invalid byte sequence — guaranteeing no panic on arbitrary
/// binary input (D-17b, threat T-02-08-02).
pub fn count_bytes(bytes: &[u8]) -> Counts {
    let mut c = Counts {
        bytes: bytes.len() as u64,
        lines: count_newlines(bytes),
        ..Counts::default()
    };

    let mut in_word = false;
    for ch in bytes.chars() {
        c.chars += 1;
        if ch.is_whitespace() {
            in_word = false;
        } else if !in_word {
            in_word = true;
            c.words += 1;
        }
    }
    c
}

fn count_newlines(bytes: &[u8]) -> u64 {
    bytes.iter().filter(|&&b| b == b'\n').count() as u64
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    // Which counts to print. If no flag given, default is -l -w -c.
    let mut want_lines = matches.get_flag("lines");
    let mut want_words = matches.get_flag("words");
    let mut want_bytes = matches.get_flag("bytes");
    let want_chars = matches.get_flag("chars");

    if !(want_lines || want_words || want_bytes || want_chars) {
        want_lines = true;
        want_words = true;
        want_bytes = true;
    }

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    // 2-pass column width: collect counts first, then pick digit width based on
    // the max value observed (GNU wc behaviour, matches Claude's Discretion in
    // 02-CONTEXT.md).
    let mut rows: Vec<(String, Counts)> = Vec::new();
    let mut total = Counts::default();
    let mut exit_code = 0;

    if operands.is_empty() {
        match count_reader(io::stdin().lock()) {
            Ok(c) => {
                total.add(&c);
                rows.push((String::new(), c));
            }
            Err(e) => {
                eprintln!("wc: stdin: {e}");
                exit_code = 1;
            }
        }
    } else {
        for op in &operands {
            if op == "-" {
                match count_reader(io::stdin().lock()) {
                    Ok(c) => {
                        total.add(&c);
                        rows.push(("-".to_string(), c));
                    }
                    Err(e) => {
                        eprintln!("wc: -: {e}");
                        exit_code = 1;
                    }
                }
                continue;
            }
            let converted = gow_core::path::try_convert_msys_path(op);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    match count_reader(reader) {
                        Ok(c) => {
                            total.add(&c);
                            rows.push((converted, c));
                        }
                        Err(e) => {
                            eprintln!("wc: {converted}: {e}");
                            exit_code = 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("wc: {converted}: {e}");
                    exit_code = 1;
                }
            }
        }
    }

    let width = compute_width(&rows, &total, want_lines, want_words, want_bytes, want_chars);

    for (label, c) in &rows {
        print_row(c, label, width, want_lines, want_words, want_bytes, want_chars);
    }

    if rows.len() > 1 {
        print_row(&total, "total", width, want_lines, want_words, want_bytes, want_chars);
    }

    exit_code
}

fn compute_width(
    rows: &[(String, Counts)],
    total: &Counts,
    want_lines: bool,
    want_words: bool,
    want_bytes: bool,
    want_chars: bool,
) -> usize {
    let mut max = 0u64;
    let mut consider = |c: &Counts| {
        if want_lines { max = max.max(c.lines); }
        if want_words { max = max.max(c.words); }
        if want_bytes { max = max.max(c.bytes); }
        if want_chars { max = max.max(c.chars); }
    };
    for (_, c) in rows {
        consider(c);
    }
    if rows.len() > 1 {
        consider(total);
    }
    max.to_string().len().max(1)
}

fn print_row(
    c: &Counts,
    label: &str,
    width: usize,
    want_lines: bool,
    want_words: bool,
    want_bytes: bool,
    want_chars: bool,
) {
    let mut parts: Vec<String> = Vec::new();
    if want_lines { parts.push(format!("{:>width$}", c.lines, width = width)); }
    if want_words { parts.push(format!("{:>width$}", c.words, width = width)); }
    if want_bytes { parts.push(format!("{:>width$}", c.bytes, width = width)); }
    if want_chars { parts.push(format!("{:>width$}", c.chars, width = width)); }
    let line = if label.is_empty() {
        parts.join(" ")
    } else {
        format!("{} {}", parts.join(" "), label)
    };
    println!("{line}");
}

fn count_reader<R: Read>(mut reader: R) -> io::Result<Counts> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(count_bytes(&buf))
}

fn uu_app() -> Command {
    Command::new("wc")
        .about("Print newline, word, and byte counts for each FILE")
        .arg(
            Arg::new("lines")
                .short('l')
                .long("lines")
                .action(ArgAction::SetTrue)
                .help("print the newline counts"),
        )
        .arg(
            Arg::new("words")
                .short('w')
                .long("words")
                .action(ArgAction::SetTrue)
                .help("print the word counts"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .action(ArgAction::SetTrue)
                .help("print the byte counts"),
        )
        .arg(
            Arg::new("chars")
                .short('m')
                .long("chars")
                .action(ArgAction::SetTrue)
                .help("print the character counts"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
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

    #[test]
    fn counts_add_accumulates() {
        let mut total = Counts::default();
        total.add(&Counts { lines: 1, words: 2, bytes: 10, chars: 10 });
        total.add(&Counts { lines: 3, words: 4, bytes: 20, chars: 20 });
        assert_eq!(total, Counts { lines: 4, words: 6, bytes: 30, chars: 30 });
    }
}
