//! `uu_chmod`: GNU `chmod` — change file permissions (FILE-10).
//!
//! Windows permission model (D-32): only the owner write bit matters.
//! It maps directly to `FILE_ATTRIBUTE_READONLY` (ON = read-only, OFF = writable).
//! Other mode bits (group/other, x/s/t) are silently ignored per CONTEXT.md
//! to avoid spurious warnings in cross-platform scripts.
//!
//! Recursion (`-R`) uses `walkdir::WalkDir` with `follow_links(false)` so that
//! symbolic links are treated as terminal entries — the link itself receives
//! the RO bit, not its target (matches GNU chmod -R behavior; mitigates T-03-15).

use std::ffi::OsString;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

/// What the parsed mode string asks us to do to a file's read-only bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOnlyTarget {
    /// `chmod 444`, `u-w`, `a-w`, `=r` — set FILE_ATTRIBUTE_READONLY ON.
    SetReadOnly,
    /// `chmod 644`, `u+w`, `+w`, `=rw` — set FILE_ATTRIBUTE_READONLY OFF.
    ClearReadOnly,
    /// Only non-owner / non-w clauses — leave RO bit alone.
    NoOpKeepCurrent,
}

/// Parse a GNU chmod mode string to the desired read-only-bit action.
///
/// Accepts:
/// - Octal: `0644`, `644`, `0000` … (1-4 octal digits)
/// - Symbolic: `[ugoa]*[+-=][rwxXst]*(,[ugoa]*[+-=][rwxXst]*)*`
///
/// Windows mapping (D-32): only the owner-write bit is honored. Other bits
/// (group, other, execute, sticky, setuid) are silently ignored — documented
/// trade-off for partial-support chmod on a platform without POSIX ACLs.
pub fn parse_mode(s: &str) -> Result<ReadOnlyTarget, String> {
    if s.is_empty() {
        return Err(format!("invalid mode: '{s}'"));
    }

    // Octal form: first char is a digit.
    if s.chars().next().unwrap().is_ascii_digit() {
        return parse_octal(s);
    }

    // Symbolic form. Evaluate clauses left-to-right; later wins when it
    // makes a definite statement (Set/Clear). NoOp clauses are skipped so
    // a stray `g+w` at the end doesn't erase an earlier `u+w` decision.
    let mut result = ReadOnlyTarget::NoOpKeepCurrent;
    for clause in s.split(',') {
        match parse_symbolic_clause(clause)? {
            ReadOnlyTarget::NoOpKeepCurrent => {}
            other => {
                result = other;
            }
        }
    }
    Ok(result)
}

fn parse_octal(s: &str) -> Result<ReadOnlyTarget, String> {
    // Accept leading 0 or not; max 4 octal digits.
    let digits: &str = s.trim_start_matches('0');
    let n: u32 = if digits.is_empty() {
        0
    } else {
        u32::from_str_radix(digits, 8).map_err(|_| format!("invalid mode: '{s}'"))?
    };
    // Owner write = bit 0o200.
    let owner_w = (n & 0o200) != 0;
    Ok(if owner_w {
        ReadOnlyTarget::ClearReadOnly
    } else {
        ReadOnlyTarget::SetReadOnly
    })
}

fn parse_symbolic_clause(clause: &str) -> Result<ReadOnlyTarget, String> {
    if clause.is_empty() {
        return Err("empty symbolic clause".to_string());
    }
    let bytes = clause.as_bytes();

    // Parse optional who-list.
    let mut i = 0;
    let mut who_all_or_u = false; // does this clause target owner?
    let mut who_specified = false;
    while i < bytes.len() && matches!(bytes[i], b'u' | b'g' | b'o' | b'a') {
        who_specified = true;
        if bytes[i] == b'u' || bytes[i] == b'a' {
            who_all_or_u = true;
        }
        i += 1;
    }
    // Default who is 'a' (all), per GNU chmod.
    if !who_specified {
        who_all_or_u = true;
    }

    if i >= bytes.len() {
        return Err(format!("invalid mode clause: '{clause}'"));
    }
    let op = bytes[i];
    if !matches!(op, b'+' | b'-' | b'=') {
        return Err(format!("invalid operator in clause: '{clause}'"));
    }
    i += 1;

    // Parse perm-list.
    let mut has_w = false;
    while i < bytes.len() {
        match bytes[i] {
            b'w' => has_w = true,
            b'r' | b'x' | b'X' | b's' | b't' => {} // silent ignore per D-32
            _ => return Err(format!("invalid permission bit: '{clause}'")),
        }
        i += 1;
    }

    // If this clause does NOT target owner (only g or only o), it's a no-op.
    if !who_all_or_u {
        return Ok(ReadOnlyTarget::NoOpKeepCurrent);
    }

    match (op, has_w) {
        (b'+', true) => Ok(ReadOnlyTarget::ClearReadOnly),
        (b'-', true) => Ok(ReadOnlyTarget::SetReadOnly),
        (b'=', true) => Ok(ReadOnlyTarget::ClearReadOnly),
        // `=` without `w` means "remove owner write": e.g. `u=r` → RO.
        (b'=', false) => Ok(ReadOnlyTarget::SetReadOnly),
        // +/- without `w` (only r/x/X/s/t): no effect on RO bit.
        (_, false) => Ok(ReadOnlyTarget::NoOpKeepCurrent),
        _ => Ok(ReadOnlyTarget::NoOpKeepCurrent),
    }
}

fn apply_readonly(path: &Path, target: ReadOnlyTarget, verbose: bool) -> std::io::Result<()> {
    match target {
        ReadOnlyTarget::NoOpKeepCurrent => {
            if verbose {
                println!("mode of '{}' retained", path.display());
            }
            Ok(())
        }
        ReadOnlyTarget::SetReadOnly | ReadOnlyTarget::ClearReadOnly => {
            let md = std::fs::metadata(path)?;
            let mut perms = md.permissions();
            let want_ro = target == ReadOnlyTarget::SetReadOnly;
            // D-32: `set_readonly` toggles FILE_ATTRIBUTE_READONLY on Windows;
            // T-03-04 mitigation: it does NOT touch the DACL, so existing ACLs
            // are preserved and we cannot widen permissions beyond the user's
            // original rights.
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(want_ro);
            std::fs::set_permissions(path, perms)?;
            if verbose {
                let label = if want_ro { "read-only" } else { "writable" };
                println!("mode of '{}' changed to {}", path.display(), label);
            }
            Ok(())
        }
    }
}

fn uu_app() -> Command {
    Command::new("chmod")
        .about("GNU chmod — change file read-only attribute (Windows D-32 subset)")
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("Recurse into directories"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Print message for each processed file"),
        )
        .arg(
            Arg::new("changes")
                .short('c')
                .long("changes")
                .action(ArgAction::SetTrue)
                .help("Like -v but only report changes (partial support on Windows — same as -v)"),
        )
        .arg(
            Arg::new("quiet")
                .short('f')
                .long("silent")
                .visible_alias("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress error messages (GNU -f)"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let recursive = matches.get_flag("recursive");
    let verbose = matches.get_flag("verbose") || matches.get_flag("changes");
    let silent = matches.get_flag("quiet");

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.len() < 2 {
        eprintln!("chmod: missing operand");
        eprintln!("usage: chmod [OPTION]... MODE FILE...");
        return 1;
    }

    let mode_str = &operands[0];
    let target = match parse_mode(mode_str) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("chmod: {e}");
            return 1;
        }
    };

    let files = &operands[1..];
    let mut exit_code = 0;

    for op in files {
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);

        if recursive && path.is_dir() {
            // D-46: walkdir traversal; follow_links(false) ensures symlinks
            // are not dereferenced (T-03-15). sort_by_file_name gives a
            // deterministic order for test reproducibility.
            for entry in walkdir::WalkDir::new(path)
                .follow_links(false)
                .sort_by_file_name()
                .into_iter()
            {
                match entry {
                    Ok(e) => {
                        if let Err(err) = apply_readonly(e.path(), target, verbose) {
                            if !silent {
                                eprintln!(
                                    "chmod: cannot access '{}': {}",
                                    e.path().display(),
                                    err
                                );
                            }
                            exit_code = 1;
                        }
                    }
                    Err(err) => {
                        if !silent {
                            let path_display = err
                                .path()
                                .map(|p| p.display().to_string())
                                .unwrap_or_default();
                            let io_msg = err
                                .io_error()
                                .map(|e| e.to_string())
                                .unwrap_or_else(|| err.to_string());
                            eprintln!("chmod: cannot access '{path_display}': {io_msg}");
                        }
                        exit_code = 1;
                    }
                }
            }
        } else if let Err(err) = apply_readonly(path, target, verbose) {
            if !silent {
                eprintln!("chmod: cannot access '{converted}': {err}");
            }
            exit_code = 1;
        }
    }

    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_octal_644_clears_ro() {
        assert_eq!(parse_mode("644").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_0644_clears_ro() {
        assert_eq!(parse_mode("0644").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_444_sets_ro() {
        assert_eq!(parse_mode("444").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_400_sets_ro() {
        assert_eq!(parse_mode("400").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_600_clears_ro() {
        assert_eq!(parse_mode("600").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_000_sets_ro() {
        assert_eq!(parse_mode("000").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_octal_777_clears_ro() {
        assert_eq!(parse_mode("777").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_octal_invalid_digit() {
        assert!(parse_mode("999").is_err());
        assert!(parse_mode("abc").is_err());
    }

    #[test]
    fn parse_symbolic_plus_w_clears() {
        assert_eq!(parse_mode("+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("u+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("a+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_symbolic_minus_w_sets() {
        assert_eq!(parse_mode("-w").unwrap(), ReadOnlyTarget::SetReadOnly);
        assert_eq!(parse_mode("u-w").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_symbolic_equal_r_sets_ro() {
        assert_eq!(parse_mode("u=r").unwrap(), ReadOnlyTarget::SetReadOnly);
    }

    #[test]
    fn parse_symbolic_equal_rw_clears() {
        assert_eq!(parse_mode("u=rw").unwrap(), ReadOnlyTarget::ClearReadOnly);
    }

    #[test]
    fn parse_symbolic_group_only_is_noop() {
        assert_eq!(parse_mode("g+w").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
        assert_eq!(parse_mode("o-w").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
    }

    #[test]
    fn parse_symbolic_x_bit_is_noop() {
        assert_eq!(parse_mode("+x").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
        assert_eq!(parse_mode("u+x").unwrap(), ReadOnlyTarget::NoOpKeepCurrent);
    }

    #[test]
    fn parse_symbolic_empty_clause_errors() {
        assert!(parse_mode("").is_err());
    }

    #[test]
    fn parse_symbolic_multi_clause_last_wins() {
        // First clause sets RO, second clears — final is clear
        assert_eq!(parse_mode("u-w,u+w").unwrap(), ReadOnlyTarget::ClearReadOnly);
        assert_eq!(parse_mode("u+w,u-w").unwrap(), ReadOnlyTarget::SetReadOnly);
    }
}
