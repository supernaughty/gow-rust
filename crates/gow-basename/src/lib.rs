//! `uu_basename`: GNU `basename` with MSYS path pre-conversion (D-26)
//! and optional suffix stripping.
//!
//! Supported flags: -a (all), -s SUFFIX (multi with suffix), -z (null terminator).
//! Reference: CONTEXT.md D-26, PATTERNS.md S4.

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    let multi = matches.get_flag("multiple");
    let zero = matches.get_flag("zero");
    let suffix: Option<String> = matches.get_one::<String>("suffix").cloned();

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("basename: missing operand");
        return 1;
    }

    // Two modes:
    //   Mode A: single operand, optional positional SUFFIX = operands[1].
    //           Only if -a and -s are both absent.
    //   Mode B: multi — implicit with -a or -s; each operand gets basenamed independently.
    let mode_multi = multi || suffix.is_some();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let terminator: &[u8] = if zero { b"\0" } else { b"\n" };

    if mode_multi {
        // -s SUFFIX applies to all operands; -a operands all processed; no positional SUFFIX allowed.
        let suf = suffix.unwrap_or_default();
        for op in &operands {
            let result = basename_with_optional_suffix(op, &suf);
            out.write_all(result.as_bytes()).ok();
            out.write_all(terminator).ok();
        }
    } else {
        // Single-arg mode. If 2 operands, operands[1] is the SUFFIX.
        let name = &operands[0];
        let suf = operands.get(1).map(|s| s.as_str()).unwrap_or("");
        let result = basename_with_optional_suffix(name, suf);
        out.write_all(result.as_bytes()).ok();
        out.write_all(terminator).ok();
    }

    0
}

fn basename_with_optional_suffix(raw: &str, suffix: &str) -> String {
    // Step 1: MSYS pre-convert (D-26).
    let converted = gow_core::path::try_convert_msys_path(raw);

    // Step 2: strip trailing separator(s) so `/foo/bar/` → `/foo/bar` and `basename` = `bar`.
    let trimmed = converted.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        // Edge: input was "/" or "\\" — return the original first separator (GNU: "/" basename = "/").
        return converted.chars().take(1).collect();
    }

    // Step 3: extract last component using Path::file_name (handles both / and \ on Windows).
    let p = Path::new(trimmed);
    let base = p
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| trimmed.to_string());

    // Step 4: strip suffix if it matches AND is not equal to the entire basename (GNU rule).
    if !suffix.is_empty() && base.len() > suffix.len() && base.ends_with(suffix) {
        base[..base.len() - suffix.len()].to_string()
    } else {
        base
    }
}

fn uu_app() -> Command {
    Command::new("basename")
        .arg(
            Arg::new("multiple")
                .short('a')
                .long("multiple")
                .action(ArgAction::SetTrue)
                .help("support multiple arguments; each is treated as a NAME"),
        )
        .arg(
            Arg::new("suffix")
                .short('s')
                .long("suffix")
                .num_args(1)
                .help("remove a trailing SUFFIX; implies -a"),
        )
        .arg(
            Arg::new("zero")
                .short('z')
                .long("zero")
                .action(ArgAction::SetTrue)
                .help("end each output line with NUL, not newline"),
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
    fn strips_trailing_slash_and_returns_last() {
        assert_eq!(basename_with_optional_suffix("/foo/bar/", ""), "bar");
    }

    #[test]
    fn msys_path_preconverted_then_basename() {
        assert_eq!(basename_with_optional_suffix("/c/Users/foo/doc.md", ""), "doc.md");
    }

    #[test]
    fn suffix_stripped_when_matches() {
        assert_eq!(basename_with_optional_suffix("foo/bar.txt", ".txt"), "bar");
    }

    #[test]
    fn suffix_not_stripped_when_equals_whole_name() {
        // GNU rule: don't strip suffix if it equals the full basename.
        assert_eq!(basename_with_optional_suffix(".txt", ".txt"), ".txt");
    }

    #[test]
    fn suffix_not_stripped_when_no_match() {
        assert_eq!(basename_with_optional_suffix("foo/bar.md", ".txt"), "bar.md");
    }

    #[test]
    fn bare_filename_is_returned_as_is() {
        assert_eq!(basename_with_optional_suffix("foo", ""), "foo");
    }
}
