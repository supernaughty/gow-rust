//! `uu_echo`: GNU `echo` with -n / -e / -E semantics.
//!
//! Entry: `pub fn uumain`. Escape parsing is delegated to `escape::write_escaped`.
//!
//! Argument handling uses an ad-hoc scanner instead of clap (CONTEXT.md D-21
//! explicitly allows this). Rationale: GNU `echo` only recognizes the three
//! short flags `-n`, `-e`, `-E` (and their combinations like `-neE`); any other
//! hyphen-prefixed argument in first position falls into one of two buckets:
//!   (a) a recognized long flag we choose to support: `--help`, `--version`;
//!   (b) everything else → treated as an argument parse error (exit 1 via
//!       `gow_core::args::parse_gnu` conventions; D-02).
//! This satisfies PLAN.md Task 2 acceptance criterion "`--bad` exits with code 1"
//! without absorbing unknown flags as positional arguments (which is what clap's
//! `trailing_var_arg(true) + allow_hyphen_values(true)` combination would do).
//!
//! Reference: RESEARCH.md Q9 (state machine) and CONTEXT.md D-21.

mod escape;

use std::ffi::OsString;
use std::io::{self, Write};

use crate::escape::{Control, write_escaped};

const HELP_TEXT: &str = "\
Usage: echo [SHORT-OPTION]... [STRING]...
  or:  echo LONG-OPTION
Echo the STRING(s) to standard output.

  -n             do not output the trailing newline
  -e             enable interpretation of backslash escapes
  -E             disable interpretation of backslash escapes (default)
      --help     display this help and exit
      --version  output version information and exit

If -e is in effect, the following sequences are recognized:

  \\\\      backslash
  \\a      alert (BEL)
  \\b      backspace
  \\c      produce no further output
  \\e      escape
  \\f      form feed
  \\n      new line
  \\r      carriage return
  \\t      horizontal tab
  \\v      vertical tab
  \\0NNN   byte with octal value NNN (1 to 3 digits)
  \\xHH    byte with hexadecimal value HH (1 to 2 digits)
";

const VERSION_TEXT: &str = concat!("echo (gow-rust) ", env!("CARGO_PKG_VERSION"));

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let mut iter = args.into_iter();
    let _argv0 = iter.next(); // skip program name

    let mut no_newline = false;
    let mut interpret_escapes = false;
    let mut body: Vec<OsString> = Vec::new();
    let mut flags_done = false;

    for arg in iter {
        if flags_done {
            body.push(arg);
            continue;
        }

        // Inspect this token as UTF-8 if possible. Non-UTF-8 tokens are
        // always positional (the three echo flags are all ASCII-only).
        let Some(s) = arg.to_str() else {
            flags_done = true;
            body.push(arg);
            continue;
        };

        // Long flags: --help, --version are honored; any other `--…` is an error.
        if let Some(long) = s.strip_prefix("--") {
            if long.is_empty() {
                // Bare `--` would be a POSIX-style end-of-options marker. GNU
                // echo does NOT honor it (it prints `--` literally). Match GNU.
                flags_done = true;
                body.push(arg);
                continue;
            }
            match long {
                "help" => {
                    print!("{HELP_TEXT}");
                    return 0;
                }
                "version" => {
                    println!("{VERSION_TEXT}");
                    return 0;
                }
                other => {
                    // Unknown long flag: argument error (D-02 → exit 1).
                    eprintln!("echo: unrecognized option '--{other}'");
                    return 1;
                }
            }
        }

        // Short flags: must be `-` followed by ONLY chars from {n, e, E}, and
        // at least one char. Anything else (e.g. `-x`, `-`, `foo`) is body.
        if let Some(short) = s.strip_prefix('-')
            && !short.is_empty()
            && short.chars().all(|c| matches!(c, 'n' | 'e' | 'E'))
        {
            for c in short.chars() {
                match c {
                    'n' => no_newline = true,
                    'e' => interpret_escapes = true,
                    'E' => interpret_escapes = false,
                    _ => unreachable!(),
                }
            }
            continue;
        }

        // Not a recognized flag → this and every later token are body.
        flags_done = true;
        body.push(arg);
    }

    // Join body with spaces. Non-UTF-8 bytes are preserved as lossy UTF-8
    // (echo output is always textual on Windows — console is UTF-8 per D-10).
    let joined: String = body
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, a)| {
            if i > 0 {
                acc.push(' ');
            }
            acc.push_str(&a.to_string_lossy());
            acc
        });
    let bytes = joined.as_bytes();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let control = if interpret_escapes {
        match write_escaped(bytes, &mut out) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("echo: {e}");
                return 1;
            }
        }
    } else {
        if let Err(e) = out.write_all(bytes) {
            eprintln!("echo: {e}");
            return 1;
        }
        Control::Continue
    };

    // Trailing newline suppression (D-21 + Q9):
    //   - `-n` suppresses always
    //   - `\c` (Control::Break) suppresses (requires -e to reach)
    let suppress_newline = no_newline || matches!(control, Control::Break);
    if !suppress_newline
        && let Err(e) = out.write_all(b"\n")
    {
        eprintln!("echo: {e}");
        return 1;
    }
    0
}
