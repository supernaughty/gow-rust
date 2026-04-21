//! `uu_pwd`: GNU `pwd` with -L (logical, default) and -P (physical) flags.
//! Handles Windows `\\?\` UNC-prefix stripping safely (see `canonical` module).
//! Reference: RESEARCH.md Q8 + CONTEXT.md D-24.

mod canonical;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};

use crate::canonical::simplify_canonical;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let physical = matches.get_flag("physical");

    let cwd = if physical {
        match std::env::current_dir().and_then(std::fs::canonicalize) {
            Ok(p) => simplify_canonical(&p),
            Err(e) => {
                eprintln!("pwd: {e}");
                return 1;
            }
        }
    } else {
        // Logical: prefer $PWD if it points at the same canonical target as current_dir;
        // otherwise fall back to current_dir. See RESEARCH.md Q8 + CONTEXT.md D-24.
        std::env::var_os("PWD")
            .map(PathBuf::from)
            .filter(|p| validate_pwd(p))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    println!("{}", cwd.display());
    0
}

/// Validate that `$PWD` names the same directory as `std::env::current_dir()`,
/// modulo canonicalization. Mitigates T-02-04-01 (malicious PWD override):
/// if the two do not canonicalize equal, or either canonicalize fails, the
/// caller falls through to `current_dir()`.
fn validate_pwd(p: &Path) -> bool {
    let Ok(cwd) = std::env::current_dir() else {
        return false;
    };
    let (Ok(pwd_canon), Ok(cwd_canon)) =
        (std::fs::canonicalize(p), std::fs::canonicalize(&cwd))
    else {
        return false;
    };
    pwd_canon == cwd_canon
}

fn uu_app() -> Command {
    Command::new("pwd")
        .arg(
            Arg::new("logical")
                .short('L')
                .action(ArgAction::SetTrue)
                .help("use PWD from environment, even if it contains symlinks (default)"),
        )
        .arg(
            Arg::new("physical")
                .short('P')
                .action(ArgAction::SetTrue)
                .help("avoid all symlinks; print canonical path"),
        )
}
