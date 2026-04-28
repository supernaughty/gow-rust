//! `uu_xargs`: GNU xargs — Windows port (R016 / XARGS-01).
//!
//! Implements -0 (NUL-separated input), -I {} (fixed replacement), -n maxargs,
//! -L maxlines. Serial-only (D-11). Per CONTEXT.md D-11, D-12.
//!
//! Exit codes (GNU/POSIX-aligned):
//!   0   = all child invocations succeeded
//!   123 = at least one child invocation exited with non-zero code
//!   124 = at least one child invocation was killed by signal (rare on Windows)
//!   125 = xargs itself failed (parse error, exec failure, IO error)
//!
//! GNU 4.4+ default: empty stdin → do NOT run command, exit 0 (--no-run-if-empty is now the
//! default). This implementation follows that behavior.
//!
//! Deferred (D-11): -P (parallel execution), -r/--no-run-if-empty toggle, -s size limit,
//! configurable -I STR (only literal "{}" per D-12).

use anyhow::{anyhow, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    name = "xargs",
    about = "GNU xargs — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = true,
)]
struct Cli {
    /// NUL-separated input (pairs with find -print0); sets stdin to binary mode on Windows
    #[arg(short = '0', long = "null", action = clap::ArgAction::SetTrue)]
    null: bool,

    /// Fixed-string replacement: literal "{}" in args is replaced with each input token (D-12).
    /// One invocation per token. Mutually exclusive with -n and -L.
    #[arg(short = 'I', long = "replace", action = clap::ArgAction::SetTrue)]
    replace: bool,

    /// Maximum arguments per command invocation
    #[arg(short = 'n', long = "max-args", value_name = "N")]
    max_args: Option<usize>,

    /// Maximum input lines per command invocation (one record per input line; N lines per batch)
    #[arg(short = 'L', long = "max-lines", value_name = "N")]
    max_lines: Option<usize>,

    /// Command and its base args (default: echo). Trailing variadic.
    #[arg(default_value = "echo", trailing_var_arg = true)]
    command_and_args: Vec<String>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("xargs: {}", e);
            return 125;
        }
    };
    match run(cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("xargs: {}", e);
            125
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    // Mutual-exclusion validation (GNU semantics)
    if cli.replace && (cli.max_args.is_some() || cli.max_lines.is_some()) {
        return Err(anyhow!("-I cannot be combined with -n or -L"));
    }
    if cli.command_and_args.is_empty() {
        return Err(anyhow!("missing command"));
    }
    let cmd = &cli.command_and_args[0];
    let base_args: Vec<String> = cli.command_and_args[1..].to_vec();

    // Binary mode stdin BEFORE any reads, when -0 is active.
    // Without this, Windows CRT text mode translates 0x1A → EOF and 0x0D 0x0A → 0x0A,
    // corrupting NUL-separated streams from `find -print0`. (T-05-xargs-01)
    if cli.null {
        set_stdin_binary_mode();
    }

    let stdin = io::stdin();
    let reader = stdin.lock();
    let tokens = tokenize_stdin(reader, cli.null);

    // GNU xargs 4.4+ default: don't run with empty input. Exit 0.
    if tokens.is_empty() {
        return Ok(0);
    }

    let mut child_codes: Vec<Option<i32>> = Vec::new();

    if cli.replace {
        // -I {} — one invocation per token, substring replacement of "{}" in each base arg.
        for tok in &tokens {
            let code = exec_with_replacement(cmd, &base_args, tok)?;
            child_codes.push(code);
        }
    } else if let Some(n) = cli.max_args {
        // -n N — at most N arguments per invocation
        for chunk in tokens.chunks(n.max(1)) {
            let code = exec_batch(cmd, &base_args, chunk)?;
            child_codes.push(code);
        }
    } else if let Some(l) = cli.max_lines {
        // -L N — N input records per command (one record = one newline-delimited line, per D-12)
        for chunk in tokens.chunks(l.max(1)) {
            let code = exec_batch(cmd, &base_args, chunk)?;
            child_codes.push(code);
        }
    } else {
        // Default: append all tokens to a single invocation.
        let code = exec_batch(cmd, &base_args, &tokens)?;
        child_codes.push(code);
    }

    Ok(aggregate_exit(&child_codes))
}

/// Set stdin to binary mode on Windows (BEFORE any reads).
///
/// The Windows CRT opens stdin in text mode by default:
///   - 0x0D 0x0A (CRLF) → 0x0A (LF)  — mangling path separators is harmless but confusing
///   - 0x1A (Ctrl-Z) → interpreted as EOF, silently truncating the stream
///   - 0x00 (NUL) is passed through in MSVC text mode but the translation above still applies
///
/// `_setmode(0, _O_BINARY)` disables all translation so NUL bytes and CRLF survive unchanged.
/// This is extern "C" (cdecl), NOT extern "system" — _setmode is a CRT function, not Win32.
#[cfg(target_os = "windows")]
fn set_stdin_binary_mode() {
    unsafe extern "C" {
        fn _setmode(fd: i32, flags: i32) -> i32;
    }
    const _O_BINARY: i32 = 0x8000;
    unsafe {
        _setmode(0, _O_BINARY);
    }
}
#[cfg(not(target_os = "windows"))]
fn set_stdin_binary_mode() {}

/// Tokenize stdin by newline (default) or NUL byte (-0 mode).
///
/// - Trailing delimiter is stripped.
/// - In newline mode, trailing CR is also stripped (CRLF Windows compatibility).
/// - Empty tokens after stripping are skipped.
/// - Non-UTF-8 byte sequences are logged to stderr and skipped.
pub fn tokenize_stdin<R: BufRead>(mut reader: R, null_delimited: bool) -> Vec<String> {
    let delimiter = if null_delimited { b'\0' } else { b'\n' };
    let mut tokens = Vec::new();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_until(delimiter, &mut buf) {
            Ok(0) => break,
            Ok(_) => {
                // Strip the trailing delimiter
                if buf.last() == Some(&delimiter) {
                    buf.pop();
                }
                // In newline mode: also strip trailing CR (handles Windows CRLF line endings)
                if !null_delimited && buf.last() == Some(&b'\r') {
                    buf.pop();
                }
                // Skip empty tokens
                if buf.is_empty() {
                    continue;
                }
                match String::from_utf8(buf.clone()) {
                    Ok(s) => tokens.push(s),
                    Err(_) => eprintln!("xargs: skipping non-UTF8 input"),
                }
            }
            Err(_) => break,
        }
    }
    tokens
}

/// Perform substring replacement of literal `{}` in `arg` with `token`.
///
/// GNU xargs -I {} semantics: every occurrence of `{}` inside the arg is replaced.
/// No shell quoting is applied — each substituted arg is passed as a single argv entry
/// to `CreateProcessW` (T-05-xargs-02: no shell re-parsing).
pub fn replace_braces(arg: &str, token: &str) -> String {
    arg.replace("{}", token)
}

/// Execute `cmd` with `base_args` + `batch` appended.
///
/// Returns `Ok(Some(code))` for a normal exit, `Ok(None)` if the process was killed by
/// a signal (rare on Windows), `Err(...)` if the process could not be spawned at all.
fn exec_batch(cmd: &str, base_args: &[String], batch: &[String]) -> Result<Option<i32>> {
    let mut all_args: Vec<String> = base_args.to_vec();
    all_args.extend_from_slice(batch);
    let status = Command::new(cmd)
        .args(&all_args)
        .status()
        .map_err(|e| anyhow!("failed to run '{}': {}", cmd, e))?;
    Ok(status.code())
}

/// Execute `cmd` with `base_args` after substituting `{}` → `token` in each base arg.
///
/// Runs exactly once per token (D-12 -I behavior).
fn exec_with_replacement(cmd: &str, base_args: &[String], token: &str) -> Result<Option<i32>> {
    let replaced: Vec<String> = base_args.iter().map(|a| replace_braces(a, token)).collect();
    let status = Command::new(cmd)
        .args(&replaced)
        .status()
        .map_err(|e| anyhow!("failed to run '{}': {}", cmd, e))?;
    Ok(status.code())
}

/// Aggregate child exit codes into a GNU-compatible xargs exit code.
///
/// | Result                       | Exit code |
/// |------------------------------|-----------|
/// | All children exited 0        | 0         |
/// | Any child exited non-zero    | 123       |
/// | Any child killed by signal   | 124       |
/// | (spawn failure handled above)| 125       |
///
/// Note: signal-killed (None) takes precedence over non-zero exit (123 < 124).
pub fn aggregate_exit(codes: &[Option<i32>]) -> i32 {
    let mut signal_killed = false;
    let mut any_failure = false;
    for c in codes {
        match c {
            None => signal_killed = true,
            Some(0) => {}
            Some(_) => any_failure = true,
        }
    }
    if signal_killed {
        124
    } else if any_failure {
        123
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- tokenize_stdin tests ---

    #[test]
    fn test_tokenize_newline() {
        let input = b"foo\nbar\nbaz\n";
        assert_eq!(tokenize_stdin(&input[..], false), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_tokenize_newline_strips_cr() {
        let input = b"foo\r\nbar\r\n";
        assert_eq!(tokenize_stdin(&input[..], false), vec!["foo", "bar"]);
    }

    #[test]
    fn test_tokenize_null() {
        let input = b"foo\0bar\0baz\0";
        assert_eq!(tokenize_stdin(&input[..], true), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_tokenize_null_preserves_newline_in_token() {
        // In NUL mode, newlines inside a token are LITERAL — not separators.
        let input = b"foo\nwith newline\0bar\0";
        assert_eq!(
            tokenize_stdin(&input[..], true),
            vec!["foo\nwith newline", "bar"]
        );
    }

    #[test]
    fn test_tokenize_skips_empty() {
        let input = b"foo\n\nbar\n";
        assert_eq!(tokenize_stdin(&input[..], false), vec!["foo", "bar"]);
    }

    // --- replace_braces tests ---

    #[test]
    fn test_replace_braces_substring() {
        assert_eq!(replace_braces("prefix/{}/suffix", "x"), "prefix/x/suffix");
    }

    #[test]
    fn test_replace_braces_multiple() {
        assert_eq!(replace_braces("{}-{}", "x"), "x-x");
    }

    #[test]
    fn test_replace_braces_none() {
        assert_eq!(replace_braces("no-brace", "x"), "no-brace");
    }

    // --- aggregate_exit tests ---

    #[test]
    fn test_aggregate_exit_all_success() {
        assert_eq!(aggregate_exit(&[Some(0), Some(0)]), 0);
    }

    #[test]
    fn test_aggregate_exit_some_failure() {
        assert_eq!(aggregate_exit(&[Some(0), Some(1)]), 123);
    }

    #[test]
    fn test_aggregate_exit_signal_dominates() {
        // Signal-killed takes precedence over non-zero exit
        assert_eq!(aggregate_exit(&[Some(1), None]), 124);
    }
}
