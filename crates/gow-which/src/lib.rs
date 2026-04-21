//! `uu_which`: GNU `which` + Windows PATHEXT hybrid search (WHICH-01, GOW #276).
//!
//! For each PATH directory:
//!   1. Try literal name (preserves GNU scripts expecting exact match)
//!   2. If not found, try name + each PATHEXT extension (`foo` → `foo.COM`, `foo.EXE`, ...)
//!
//! Reference: RESEARCH.md Q6, CONTEXT.md D-18.

mod pathext;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let all = matches.get_flag("all");

    let names: Vec<String> = matches
        .get_many::<String>("names")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if names.is_empty() {
        eprintln!("which: missing argument");
        return 1;
    }

    let mut exit_code = 0;
    for name in &names {
        let hits = find(OsStr::new(name), all);
        if hits.is_empty() {
            // GNU which: "no foo in (PATH)" to stderr, exit 1
            eprintln!(
                "which: no {name} in ({})",
                std::env::var_os("PATH")
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            exit_code = 1;
        } else {
            for hit in hits {
                println!("{}", hit.display());
            }
        }
    }

    exit_code
}

/// Hybrid PATH + PATHEXT search. Returns a Vec of absolute paths.
/// If `all` is false, returns at most 1 hit (the first match across all PATH directories).
pub fn find(name: &OsStr, all: bool) -> Vec<PathBuf> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let pathext = pathext::load_pathext();
    let mut hits: Vec<PathBuf> = Vec::new();

    for dir in std::env::split_paths(&path_var) {
        // Phase 1: literal match (no extension appended)
        let literal = dir.join(name);
        if is_executable_file(&literal) {
            hits.push(literal);
            if !all {
                return hits;
            }
        }

        // Phase 2: PATHEXT expansions
        for ext in &pathext {
            let mut candidate_os: OsString = dir.join(name).into_os_string();
            candidate_os.push(ext);
            let candidate = PathBuf::from(candidate_os);
            if is_executable_file(&candidate) {
                hits.push(candidate);
                if !all {
                    return hits;
                }
            }
        }
    }

    hits
}

fn is_executable_file(p: &Path) -> bool {
    // Windows has no "executable" bit; we check: exists AND is a regular file.
    match std::fs::metadata(p) {
        Ok(m) => m.is_file(),
        Err(_) => false,
    }
}

fn uu_app() -> Command {
    Command::new("which")
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("print all matching executables in PATH, not just the first"),
        )
        .arg(
            Arg::new("names")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}
