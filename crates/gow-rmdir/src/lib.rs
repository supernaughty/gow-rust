//! `uu_rmdir`: GNU `rmdir` with `-p` parent walk (CONTEXT.md D-28, RESEARCH.md Q5).
//!
//! Plain `rmdir path` removes a single empty directory. `rmdir -p a/b/c` removes
//! `a/b/c`, then `a/b`, then `a`, stopping at the first ancestor that is not
//! empty (still returning exit 0 — POSIX/GNU semantics). The manual loop is
//! required because `std::fs` exposes no parent-walk helper.
//!
//! MSYS pre-convert (D-26) is applied per operand so `/c/Users/old` becomes
//! `C:\Users\old` before the `remove_dir` call.
//!
//! The `ErrorKind::DirectoryNotEmpty` variant stabilized in Rust 1.83; we
//! require 1.85, so the enum variant is the primary detection path on both
//! Windows (ERROR_DIR_NOT_EMPTY = 145) and Unix (ENOTEMPTY = 39). We also
//! compare raw OS codes as defense-in-depth per RESEARCH.md Q5.

use std::ffi::OsString;
use std::io;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let parents = matches.get_flag("parents");
    let verbose = matches.get_flag("verbose");

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("rmdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    for op in &operands {
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);

        let result = if parents {
            rmdir_parents(path, verbose)
        } else {
            remove_one(path, verbose)
        };

        if let Err(e) = result {
            eprintln!("rmdir: failed to remove '{converted}': {e}");
            exit_code = 1;
        }
    }

    exit_code
}

/// Remove a single directory and, when `verbose`, log the removal in GNU format.
fn remove_one(path: &Path, verbose: bool) -> io::Result<()> {
    std::fs::remove_dir(path)?;
    if verbose {
        println!("rmdir: removing directory, '{}'", path.display());
    }
    Ok(())
}

/// `rmdir -p` parent-walk loop (D-28). Removes the leaf first, then walks up
/// through each parent and removes it iff it is empty; stops (returning Ok)
/// at the first parent that is non-empty, preserving POSIX semantics.
fn rmdir_parents(path: &Path, verbose: bool) -> io::Result<()> {
    remove_one(path, verbose)?;

    let mut current = path.parent();
    while let Some(p) = current {
        if p.as_os_str().is_empty() {
            // Empty Path means we reached the top (e.g. relative path "a"'s
            // parent is ""). Stop — nothing left to remove.
            break;
        }
        match std::fs::remove_dir(p) {
            Ok(()) => {
                if verbose {
                    println!("rmdir: removing directory, '{}'", p.display());
                }
            }
            Err(e) if is_not_empty(&e) => break,
            Err(e) => return Err(e),
        }
        current = p.parent();
    }
    Ok(())
}

/// Detect the platform-specific "directory not empty" error.
///
/// Rust 1.85 exposes `ErrorKind::DirectoryNotEmpty` on both Windows and Unix,
/// but we also compare the raw OS error code as a defense-in-depth layer in
/// case a future std release remaps the error mapping (RESEARCH.md Q5).
fn is_not_empty(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::DirectoryNotEmpty {
        return true;
    }
    #[cfg(windows)]
    {
        // ERROR_DIR_NOT_EMPTY
        e.raw_os_error() == Some(145)
    }
    #[cfg(not(windows))]
    {
        // ENOTEMPTY on Linux/macOS
        e.raw_os_error() == Some(39)
    }
}

fn uu_app() -> Command {
    Command::new("rmdir")
        .arg(
            Arg::new("parents")
                .short('p')
                .long("parents")
                .action(ArgAction::SetTrue)
                .help(
                    "remove DIRECTORY and its ancestors; e.g., 'rmdir -p a/b/c' is \
                     equivalent to 'rmdir a/b/c a/b a'",
                ),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("output a diagnostic for every directory processed"),
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
    fn is_not_empty_detects_rust_kind() {
        let e = io::Error::from(io::ErrorKind::DirectoryNotEmpty);
        assert!(is_not_empty(&e));
    }

    #[test]
    fn is_not_empty_rejects_not_found() {
        let e = io::Error::from(io::ErrorKind::NotFound);
        assert!(!is_not_empty(&e));
    }
}
