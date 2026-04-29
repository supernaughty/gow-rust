//! `uu_du`: GNU du — disk usage using walkdir + 1K-block / human-readable output (U-08).

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "du",
    about = "GNU du — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Display only a total for each argument
    #[arg(short = 's', long = "summarize", action = ArgAction::SetTrue)]
    summarize: bool,

    /// Print sizes in human-readable format (e.g. 1K, 234M, 2G)
    #[arg(short = 'h', long = "human-readable", action = ArgAction::SetTrue)]
    human: bool,

    /// Write counts for all files, not just directories
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
    all: bool,

    /// Print sizes in bytes
    #[arg(short = 'b', long = "bytes", action = ArgAction::SetTrue)]
    bytes: bool,

    /// Maximum traversal depth (0 = print only the args themselves)
    #[arg(short = 'd', long = "max-depth")]
    max_depth: Option<usize>,

    /// Files/directories (default: ".")
    paths: Vec<String>,
}

/// Human-readable formatting (GNU binary SI units: 1K = 1024).
/// Pattern 5 from RESEARCH.md.
fn human_readable(bytes: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("E", 1u64 << 60),
        ("P", 1u64 << 50),
        ("T", 1u64 << 40),
        ("G", 1u64 << 30),
        ("M", 1u64 << 20),
        ("K", 1u64 << 10),
    ];
    for (suffix, factor) in UNITS {
        if bytes >= *factor {
            let val = bytes as f64 / *factor as f64;
            return if val < 10.0 {
                format!("{:.1}{}", val, suffix)
            } else {
                format!("{:.0}{}", val, suffix)
            };
        }
    }
    format!("{}", bytes)
}

/// Format file size according to the active flags.
fn format_size(bytes: u64, cli: &Cli) -> String {
    if cli.human {
        human_readable(bytes)
    } else if cli.bytes {
        format!("{}", bytes)
    } else {
        // Default: 1K blocks, rounded up
        format!("{}", (bytes + 1023) / 1024)
    }
}

/// Recursively sum file sizes under `path` without following symlinks.
/// T-10-05-01: follow_links(false) prevents cyclic-symlink infinite loops.
fn dir_usage_recursive(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Emit du output for a single root path.
fn run_one(cli: &Cli, root: &Path, exit: &mut i32, out: &mut impl Write) {
    if !root.exists() {
        eprintln!(
            "du: cannot access '{}': No such file or directory",
            root.display()
        );
        *exit = 1;
        return;
    }

    if cli.summarize {
        let total = dir_usage_recursive(root);
        let _ = writeln!(out, "{}\t{}", format_size(total, cli), root.display());
        return;
    }

    // Non-summarize: emit per-directory totals up to max_depth.
    // WalkDir.max_depth(0) = only the root entry; max_depth(1) = root + direct children, etc.
    // GNU du --max-depth=0 shows only the root, so we pass max_depth directly as WalkDir depth.
    let wd_max_depth = cli.max_depth.unwrap_or(usize::MAX);
    let walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(wd_max_depth);

    for entry in walker {
        match entry {
            Ok(e) => {
                let is_dir = e.file_type().is_dir();
                let is_file = e.file_type().is_file();
                if is_dir {
                    // Compute total size of all files under this directory
                    let total = dir_usage_recursive(e.path());
                    let _ = writeln!(out, "{}\t{}", format_size(total, cli), e.path().display());
                } else if cli.all && is_file {
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    let _ = writeln!(out, "{}\t{}", format_size(size, cli), e.path().display());
                }
            }
            Err(err) => {
                eprintln!("du: {err}");
                *exit = 1;
            }
        }
    }
}

fn run(cli: &Cli) -> i32 {
    // GNU: --summarize and --max-depth are mutually exclusive
    if cli.summarize && cli.max_depth.is_some() {
        eprintln!("du: cannot both summarize and show all entries");
        return 1;
    }

    let paths: Vec<String> = if cli.paths.is_empty() {
        vec![".".to_string()]
    } else {
        cli.paths.clone()
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0i32;

    for p_str in &paths {
        let p_converted = gow_core::path::try_convert_msys_path(p_str);
        let root = Path::new(&p_converted);
        run_one(cli, root, &mut exit_code, &mut out);
    }

    exit_code
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("du: {e}");
            return 2;
        }
    };
    run(&cli)
}
