//! `uu_dirname`: GNU `dirname` with MSYS path pre-conversion (D-26).
//!
//! Multi-arg: each operand gets dirname'd and printed on its own line (GNU default).
//! Flag: -z for NUL-terminated output.

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let zero = matches.get_flag("zero");

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("dirname: missing operand");
        return 1;
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let terminator: &[u8] = if zero { b"\0" } else { b"\n" };

    for op in &operands {
        let result = dirname_of(op);
        out.write_all(result.as_bytes()).ok();
        out.write_all(terminator).ok();
    }

    0
}

fn dirname_of(raw: &str) -> String {
    // Step 1: MSYS pre-convert (D-26).
    let converted = gow_core::path::try_convert_msys_path(raw);

    // Edge: empty input. GNU `dirname ""` prints `.`.
    if converted.is_empty() {
        return ".".to_string();
    }

    // Step 2: trim trailing separators (GNU: dirname of "foo/" is ".").
    // Edge: "/" alone should return "/"; "\" alone should return "\".
    let trimmed_end = converted.trim_end_matches(['/', '\\']);
    let working = if trimmed_end.is_empty() {
        // Entire input was slashes only — return the first separator character.
        return converted.chars().take(1).collect();
    } else {
        trimmed_end
    };

    // Step 3: parent via std::path::Path.
    let p = Path::new(working);
    match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_string_lossy().into_owned(),
        _ => ".".to_string(),
    }
}

fn uu_app() -> Command {
    Command::new("dirname")
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
    fn dirname_of_nested_path() {
        assert_eq!(dirname_of("foo/bar.txt"), "foo");
    }

    #[test]
    fn dirname_of_bare_file_is_dot() {
        assert_eq!(dirname_of("foo"), ".");
    }

    #[test]
    fn dirname_of_msys_path_converted() {
        // /c/Users/foo/doc.md → C:\Users\foo\doc.md → C:\Users\foo
        assert_eq!(dirname_of("/c/Users/foo/doc.md"), r"C:\Users\foo");
    }

    #[test]
    fn dirname_of_trailing_slash() {
        // "foo/bar/" → dirname = "foo"
        assert_eq!(dirname_of("foo/bar/"), "foo");
    }

    #[test]
    fn dirname_empty_string_is_dot() {
        assert_eq!(dirname_of(""), ".");
    }
}
