//! `uu_head`: GNU `head` — output first N lines or bytes of files (TEXT-01).
//!
//! Raw-byte policy (D-48): `-n` counts `b'\n'` terminators; `-c` is
//! byte-exact (may split multi-byte UTF-8 characters — matches GNU).
//!
//! Numeric shorthand (D-05): `head -5 file` works via a pre-parse step
//! that rewrites leading `-N` into `-n N` before handing to clap.
//!
//! Flags supported:
//! - `-n NUM` / `--lines NUM`   first N lines (default 10)
//! - `-c NUM` / `--bytes NUM`   first N bytes
//! - `-q` / `--quiet` / `--silent`  never print `==> file <==` headers
//! - `-v` / `--verbose`          always print headers (even for single file)
//! - `-NUM` shorthand (e.g. `-5`) rewrites to `-n 5`.

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use clap::{Arg, ArgAction, Command};

/// Build the clap `Command` describing head's flags.
fn uu_app() -> Command {
    Command::new("head")
        .about("GNU head — output the first part of files")
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .num_args(1)
                .default_value("10")
                .help("print the first NUM lines instead of the first 10"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .num_args(1)
                .help("print the first NUM bytes"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .visible_alias("silent")
                .action(ArgAction::SetTrue)
                .help("never print headers giving file names"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("always print headers giving file names"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

/// Which counting mode to use: N lines or N bytes.
#[derive(Debug, Clone, Copy)]
enum Mode {
    Lines(u64),
    Bytes(u64),
}

/// Rewrite `-N` (digits only) as `-n N` in-place in the argv vector, so
/// clap's standard parser sees the canonical form. GNU's "historical"
/// shorthand (D-05).
///
/// Rules:
/// - Only argv tokens matching exactly `-<digits>+` are rewritten.
/// - Tokens like `-n`, `-c`, `--lines=5`, `-n5` are left alone.
/// - Once an `--` end-of-options marker is seen, rewriting stops.
fn expand_numeric_shorthand(args: &mut Vec<OsString>) {
    let mut i = 0;
    while i < args.len() {
        let s = args[i].to_string_lossy();
        if s == "--" {
            break;
        }
        // Match -<digits>+ exactly. Skip -c / -n / --long and mixed forms.
        if s.len() >= 2 && s.starts_with('-') && !s.starts_with("--") {
            let tail = &s[1..];
            if tail.chars().all(|c| c.is_ascii_digit()) {
                let num = tail.to_string();
                args[i] = OsString::from("-n");
                args.insert(i + 1, OsString::from(num));
                i += 2;
                continue;
            }
        }
        i += 1;
    }
}

/// Parse a decimal count string. v1 accepts bare digits only — GNU suffixes
/// (`k`, `M`, `G`) are out of scope for this plan.
fn parse_num(s: &str) -> Result<u64, String> {
    s.parse::<u64>()
        .map_err(|_| format!("invalid number of lines: '{s}'"))
}

/// Copy up to `n` newline-terminated lines from `reader` to `writer`.
///
/// `BufRead::read_until(b'\n', ...)` reads exactly one physical line per call,
/// including the trailing `\n` when present. The final line may be
/// unterminated; it is still emitted verbatim.
fn read_n_lines<R: Read, W: Write>(reader: R, writer: &mut W, n: u64) -> io::Result<()> {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::with_capacity(8192);
    for _ in 0..n {
        buf.clear();
        let read = reader.read_until(b'\n', &mut buf)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buf)?;
    }
    Ok(())
}

/// Copy up to `n` raw bytes from `reader` to `writer`. Byte-exact: may split a
/// multi-byte UTF-8 character (D-48).
fn read_n_bytes<R: Read, W: Write>(reader: R, writer: &mut W, n: u64) -> io::Result<()> {
    let mut reader = BufReader::new(reader);
    let mut take = reader.by_ref().take(n);
    io::copy(&mut take, writer)?;
    Ok(())
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // Pre-process: expand `-5` into `-n 5` so clap can parse cleanly.
    let mut argv: Vec<OsString> = args.into_iter().collect();
    expand_numeric_shorthand(&mut argv);

    let matches = gow_core::args::parse_gnu(uu_app(), argv);

    let mode = if let Some(bytes_str) = matches.get_one::<String>("bytes") {
        match parse_num(bytes_str) {
            Ok(n) => Mode::Bytes(n),
            Err(e) => {
                eprintln!("head: {e}");
                return 1;
            }
        }
    } else {
        let lines_str = matches.get_one::<String>("lines").expect("default set");
        match parse_num(lines_str) {
            Ok(n) => Mode::Lines(n),
            Err(e) => {
                eprintln!("head: {e}");
                return 1;
            }
        }
    };

    let quiet = matches.get_flag("quiet");
    let verbose = matches.get_flag("verbose");

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    let mut stdout = io::stdout().lock();
    let mut exit_code = 0;

    // Header rules (GNU): multi-file → headers by default. Single file / stdin →
    // no header unless -v. `-q` forces no headers.
    let multi = operands.len() > 1;
    let show_headers = !quiet && (multi || verbose);

    // No operands: read stdin. Show header only if -v forced it.
    if operands.is_empty() {
        if show_headers {
            let _ = writeln!(stdout, "==> standard input <==");
        }
        let res = match mode {
            Mode::Lines(n) => read_n_lines(io::stdin().lock(), &mut stdout, n),
            Mode::Bytes(n) => read_n_bytes(io::stdin().lock(), &mut stdout, n),
        };
        if let Err(e) = res {
            eprintln!("head: error reading standard input: {e}");
            exit_code = 1;
        }
        return exit_code;
    }

    let mut first = true;
    for op in &operands {
        if show_headers {
            if !first {
                let _ = writeln!(stdout);
            }
            let label = if op == "-" { "standard input" } else { op.as_str() };
            let _ = writeln!(stdout, "==> {label} <==");
        }
        first = false;

        if op == "-" {
            let res = match mode {
                Mode::Lines(n) => read_n_lines(io::stdin().lock(), &mut stdout, n),
                Mode::Bytes(n) => read_n_bytes(io::stdin().lock(), &mut stdout, n),
            };
            if let Err(e) = res {
                eprintln!("head: error reading standard input: {e}");
                exit_code = 1;
            }
            continue;
        }

        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);
        match File::open(path) {
            Ok(f) => {
                let res = match mode {
                    Mode::Lines(n) => read_n_lines(f, &mut stdout, n),
                    Mode::Bytes(n) => read_n_bytes(f, &mut stdout, n),
                };
                if let Err(e) = res {
                    eprintln!("head: error reading '{converted}': {e}");
                    exit_code = 1;
                }
            }
            Err(e) => {
                eprintln!("head: cannot open '{converted}' for reading: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorthand_rewrites_neg_digits() {
        let mut a = vec![
            OsString::from("head"),
            OsString::from("-5"),
            OsString::from("file"),
        ];
        expand_numeric_shorthand(&mut a);
        assert_eq!(
            a,
            vec![
                OsString::from("head"),
                OsString::from("-n"),
                OsString::from("5"),
                OsString::from("file"),
            ]
        );
    }

    #[test]
    fn shorthand_leaves_n_flag_alone() {
        let mut a = vec![
            OsString::from("head"),
            OsString::from("-n"),
            OsString::from("5"),
        ];
        expand_numeric_shorthand(&mut a);
        assert_eq!(
            a,
            vec![
                OsString::from("head"),
                OsString::from("-n"),
                OsString::from("5"),
            ]
        );
    }

    #[test]
    fn shorthand_leaves_c_flag_alone() {
        let mut a = vec![
            OsString::from("head"),
            OsString::from("-c"),
            OsString::from("10"),
        ];
        expand_numeric_shorthand(&mut a);
        assert_eq!(a.len(), 3);
        assert_eq!(a[1], OsString::from("-c"));
        assert_eq!(a[2], OsString::from("10"));
    }

    #[test]
    fn shorthand_stops_at_double_dash() {
        let mut a = vec![
            OsString::from("head"),
            OsString::from("--"),
            OsString::from("-5"),
        ];
        expand_numeric_shorthand(&mut a);
        // -5 after -- is a filename, don't rewrite.
        assert_eq!(a.len(), 3);
        assert_eq!(a[0], OsString::from("head"));
        assert_eq!(a[1], OsString::from("--"));
        assert_eq!(a[2], OsString::from("-5"));
    }

    #[test]
    fn shorthand_leaves_mixed_form_alone() {
        // -n5 (digit glued onto -n) is already clap-parseable; don't touch it.
        let mut a = vec![OsString::from("head"), OsString::from("-n5")];
        expand_numeric_shorthand(&mut a);
        assert_eq!(a.len(), 2);
        assert_eq!(a[1], OsString::from("-n5"));
    }

    #[test]
    fn read_n_lines_under_count() {
        let input: &[u8] = b"a\nb\n";
        let mut out = Vec::new();
        read_n_lines(input, &mut out, 5).unwrap();
        assert_eq!(out, b"a\nb\n");
    }

    #[test]
    fn read_n_lines_over_count() {
        let input: &[u8] = b"a\nb\nc\nd\n";
        let mut out = Vec::new();
        read_n_lines(input, &mut out, 2).unwrap();
        assert_eq!(out, b"a\nb\n");
    }

    #[test]
    fn read_n_lines_zero() {
        let input: &[u8] = b"a\nb\n";
        let mut out = Vec::new();
        read_n_lines(input, &mut out, 0).unwrap();
        assert_eq!(out, b"");
    }

    #[test]
    fn read_n_lines_unterminated_last_line() {
        // Final line has no trailing newline — still emitted verbatim.
        let input: &[u8] = b"a\nb";
        let mut out = Vec::new();
        read_n_lines(input, &mut out, 5).unwrap();
        assert_eq!(out, b"a\nb");
    }

    #[test]
    fn read_n_bytes_exact() {
        let input: &[u8] = b"hello world";
        let mut out = Vec::new();
        read_n_bytes(input, &mut out, 5).unwrap();
        assert_eq!(out, b"hello");
    }

    #[test]
    fn read_n_bytes_over() {
        let input: &[u8] = b"hi";
        let mut out = Vec::new();
        read_n_bytes(input, &mut out, 100).unwrap();
        assert_eq!(out, b"hi");
    }

    #[test]
    fn read_n_bytes_zero() {
        let input: &[u8] = b"hi";
        let mut out = Vec::new();
        read_n_bytes(input, &mut out, 0).unwrap();
        assert_eq!(out, b"");
    }

    #[test]
    fn parse_num_valid() {
        assert_eq!(parse_num("10").unwrap(), 10);
        assert_eq!(parse_num("0").unwrap(), 0);
    }

    #[test]
    fn parse_num_invalid() {
        assert!(parse_num("abc").is_err());
        assert!(parse_num("").is_err());
        assert!(parse_num("-5").is_err());
    }
}
