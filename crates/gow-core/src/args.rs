//! GNU-compatible argument parsing wrapper around clap 4.
//!
//! The key problem: clap exits with code 2 on argument errors; GNU tools
//! must exit with code 1. This module wraps `try_get_matches_from()` to
//! intercept and remap the exit code.
//!
//! Also ensures:
//! - Option permutation: `cmd file -flag` == `cmd -flag file` (D-03)
//! - `--` end-of-options: arguments after `--` are always positional (D-04)
//! - Numeric shorthand handled by individual utilities via clap value_parser (D-05)
//!
//! Covers: FOUND-03, D-01, D-02, D-03, D-04

use clap::error::ErrorKind;
use clap::{ArgMatches, Command};

/// Parse arguments GNU-style.
///
/// Differences from plain `cmd.get_matches_from()`:
/// - Exits with code **1** (not 2) on argument errors (D-02)
/// - Error message format: `{binary}: {error}` (D-11 preview)
/// - Relies on clap 4's default GNU-style option permutation (D-03).
///   We intentionally do NOT set `allow_hyphen_values(true)` on the Command:
///   at the Command level it propagates to every positional-accepting arg and
///   ends up absorbing real flags (e.g. `--verbose` after a positional file).
///
/// # Usage
/// ```no_run
/// use clap::Command;
/// use gow_core::args::parse_gnu;
///
/// let cmd = Command::new("myutil")
///     .arg(clap::Arg::new("verbose").short('v').long("verbose").action(clap::ArgAction::SetTrue));
/// let matches = parse_gnu(cmd, std::env::args_os());
/// ```
pub fn parse_gnu(
    cmd: Command,
    args: impl IntoIterator<Item = std::ffi::OsString>,
) -> ArgMatches {
    // Derive the binary name from argv[0] for error messages.
    // Use a snapshot of the first arg before consuming the iterator.
    let args: Vec<std::ffi::OsString> = args.into_iter().collect();
    let bin = args
        .first()
        .and_then(|s| {
            std::path::Path::new(s)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "gow".to_owned());

    // GNU option permutation (D-03): clap 4 permits options after positional args
    // by default — this is actually GNU-style permutation, not POSIX. We do NOT set
    // `allow_hyphen_values(true)` at the Command level because that turns every
    // positional-accepting argument into a "hyphen-swallower", which breaks
    // permutation by making `file.txt --verbose` absorb `--verbose` as another file.
    //
    // `allow_negative_numbers(true)` IS safe — it only affects numeric arguments
    // (e.g., `head -5`), allowing negative numbers to not be misread as flags.
    // Numeric shorthand (D-05) is handled per-utility via custom value_parser.
    let cmd = cmd; // .allow_negative_numbers(true);

    cmd.try_get_matches_from(args).unwrap_or_else(|e| {
        match e.kind() {
            // --help and --version are clap-routed as errors but should exit 0
            // and print to stdout per GNU convention.
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                // clap formats the help/version text into the error itself;
                // printing it out reproduces `--help`/`--version` output.
                print!("{e}");
                std::process::exit(0);
            }
            // Help-on-missing-required-arg is emitted by clap when a required
            // argument is missing and help would be useful. Treat as usage error
            // (exit 1 per D-02).
            _ => {
                // Print to stderr in GNU format: "{binary}: {error}" (D-11)
                // clap's Display impl already formats the error message body.
                eprintln!("{bin}: {e}");
                // GNU convention: bad arguments = exit code 1 (not clap default 2) (D-02)
                std::process::exit(1);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction};

    fn make_test_cmd() -> Command {
        Command::new("testutil")
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("file").action(ArgAction::Append))
    }

    #[test]
    fn test_double_dash_makes_remaining_positional() {
        // After `--`, flags are treated as positional values (D-04)
        let cmd = make_test_cmd();
        let matches = parse_gnu(
            cmd,
            ["testutil", "--", "--verbose"].map(std::ffi::OsString::from),
        );
        // --verbose after -- should be treated as a file argument, not a flag
        let files: Vec<&str> = matches
            .get_many::<String>("file")
            .unwrap_or_default()
            .map(|s| s.as_str())
            .collect();
        assert!(
            files.contains(&"--verbose"),
            "Expected --verbose to be a positional arg after --, got: {files:?}"
        );
        assert!(
            !matches.get_flag("verbose"),
            "--verbose after -- should not set the verbose flag"
        );
    }

    #[test]
    fn test_option_permutation_flag_after_positional() {
        // GNU permutation: flag after positional must still work (D-03)
        // `testutil file.txt --verbose` should parse --verbose as the flag
        let cmd = make_test_cmd();
        let matches = parse_gnu(
            cmd,
            ["testutil", "file.txt", "--verbose"].map(std::ffi::OsString::from),
        );
        assert!(
            matches.get_flag("verbose"),
            "--verbose after positional should still be parsed as a flag"
        );
    }
}
