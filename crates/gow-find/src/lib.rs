//! `uu_find`: GNU find — Windows port (R015 / FIND-01, FIND-02, FIND-03).
//!
//! MINIMAL BOOTSTRAP IMPLEMENTATION for plan 05-03 cross-binary pipeline test.
//! Plan 05-02 replaces this body with the full implementation.
//!
//! Supported flags (minimal set for pipeline integration):
//!   PATH          starting path for traversal (default: .)
//!   -name PATTERN glob pattern filter
//!   -type f       files only (type = f/d/l accepted; only f used in tests)
//!   -print0       NUL-separated output (binary-safe for xargs -0 pipeline)
//!   -print        newline-separated output (default)
//!   -maxdepth N   limit directory recursion depth
//!
//! This minimal implementation uses walkdir + globset to traverse directories,
//! applying -name and -type filters, and outputs paths with the chosen delimiter.

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::ffi::OsString;
use std::io::{self, Write};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "find",
    about = "GNU find — Windows port (minimal bootstrap for 05-03 pipeline test).",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = true,
    allow_hyphen_values = true,
)]
struct Cli {
    /// Starting path (default: current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Remaining arguments parsed manually (find's non-standard arg ordering)
    #[arg(trailing_var_arg = true)]
    rest: Vec<String>,
}

struct FindOptions {
    name_patterns: Vec<String>,
    file_type: Option<char>, // 'f', 'd', 'l'
    print0: bool,
    max_depth: Option<usize>,
}

fn parse_rest(rest: &[String]) -> FindOptions {
    let mut opts = FindOptions {
        name_patterns: Vec::new(),
        file_type: None,
        print0: false,
        max_depth: None,
    };
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "-name" if i + 1 < rest.len() => {
                opts.name_patterns.push(rest[i + 1].clone());
                i += 2;
            }
            "-type" if i + 1 < rest.len() => {
                opts.file_type = rest[i + 1].chars().next();
                i += 2;
            }
            "-print0" => {
                opts.print0 = true;
                i += 1;
            }
            "-print" => {
                opts.print0 = false;
                i += 1;
            }
            "-maxdepth" if i + 1 < rest.len() => {
                if let Ok(n) = rest[i + 1].parse::<usize>() {
                    opts.max_depth = Some(n);
                }
                i += 2;
            }
            _ => { i += 1; } // skip unknown flags
        }
    }
    opts
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => { eprintln!("find: {}", e); return 1; }
    };
    match run(cli) {
        Ok(c) => c,
        Err(e) => { eprintln!("find: {}", e); 1 }
    }
}

fn run(cli: Cli) -> Result<i32> {
    let opts = parse_rest(&cli.rest);

    // Compile name patterns into a GlobSet for efficient matching
    let glob_set: Option<GlobSet> = if opts.name_patterns.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for pat in &opts.name_patterns {
            builder.add(Glob::new(pat)?);
        }
        Some(builder.build()?)
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let walker = {
        let w = WalkDir::new(&cli.path).follow_links(false);
        if let Some(d) = opts.max_depth {
            w.max_depth(d)
        } else {
            w
        }
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        // -type filter
        if let Some(t) = opts.file_type {
            let ft = entry.file_type();
            let matches_type = match t {
                'f' => ft.is_file(),
                'd' => ft.is_dir(),
                'l' => ft.is_symlink(),
                _ => true,
            };
            if !matches_type {
                continue;
            }
        }

        // -name filter (matches against the final path component only, like GNU find)
        if let Some(gs) = &glob_set {
            let file_name = entry.file_name().to_string_lossy();
            if !gs.is_match(file_name.as_ref()) {
                continue;
            }
        }

        let path_str = entry.path().to_string_lossy();
        if opts.print0 {
            out.write_all(path_str.as_bytes())?;
            out.write_all(b"\0")?;
        } else {
            writeln!(out, "{}", path_str)?;
        }
    }

    Ok(0)
}
