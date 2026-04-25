use anyhow::{Context, Result};
use clap::Parser;
use similar::{ChangeTag, TextDiff};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "diff",
    about = "GNU diff — Windows port.",
    version = env!("CARGO_PKG_VERSION")
)]
struct Args {
    /// Output NUM (default 3) lines of unified context
    #[arg(
        short = 'u',
        long = "unified",
        value_name = "NUM",
        num_args = 0..=1,
        default_missing_value = "3"
    )]
    unified: Option<usize>,

    /// Output NUM lines of unified context (explicit -U NUM form)
    #[arg(short = 'U', value_name = "NUM", conflicts_with = "unified")]
    unified_explicit: Option<usize>,

    /// Recursively compare any subdirectories found
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Treat absent files as empty
    #[arg(short = 'N', long = "new-file")]
    new_file: bool,

    /// First file or directory
    file1: PathBuf,

    /// Second file or directory
    file2: PathBuf,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args: Vec<OsString> = args.into_iter().collect();
    let parsed = match Args::try_parse_from(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    match run(parsed) {
        Ok(0) => 0,
        Ok(_) => 1,
        Err(e) => {
            eprintln!("diff: {}", e);
            2
        }
    }
}

fn context_lines(args: &Args) -> usize {
    // -U NUM takes priority over -u NUM
    if let Some(n) = args.unified_explicit {
        return n;
    }
    if let Some(n) = args.unified {
        return n;
    }
    // Default: 3 lines of context (matches GNU diff default)
    3
}

fn run(args: Args) -> Result<i32> {
    let ctx = context_lines(&args);

    if args.file1.is_dir() && args.file2.is_dir() {
        if args.recursive {
            diff_dirs_recursive(&args.file1, &args.file2, ctx, args.new_file)
        } else {
            anyhow::bail!(
                "diff: {}: Is a directory",
                args.file1.display()
            );
        }
    } else {
        let had_diff = diff_files(&args.file1, &args.file2, ctx, args.new_file)?;
        Ok(if had_diff { 1 } else { 0 })
    }
}

fn diff_dirs_recursive(dir1: &Path, dir2: &Path, context: usize, new_file: bool) -> Result<i32> {
    use std::collections::BTreeSet;

    // Collect relative paths in dir1
    let mut paths1: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in WalkDir::new(dir1).min_depth(1) {
        let entry = entry.with_context(|| format!("diff: error walking {}", dir1.display()))?;
        if entry.file_type().is_file() {
            if let Ok(rel) = entry.path().strip_prefix(dir1) {
                paths1.insert(rel.to_path_buf());
            }
        }
    }

    // Collect relative paths in dir2
    let mut paths2: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in WalkDir::new(dir2).min_depth(1) {
        let entry = entry.with_context(|| format!("diff: error walking {}", dir2.display()))?;
        if entry.file_type().is_file() {
            if let Ok(rel) = entry.path().strip_prefix(dir2) {
                paths2.insert(rel.to_path_buf());
            }
        }
    }

    let all_paths: BTreeSet<PathBuf> = paths1.union(&paths2).cloned().collect();
    let mut any_diff = false;

    for rel in &all_paths {
        let p1 = dir1.join(rel);
        let p2 = dir2.join(rel);

        if !paths1.contains(rel) {
            eprintln!("Only in {}: {}", dir2.display(), rel.display());
            any_diff = true;
        } else if !paths2.contains(rel) {
            eprintln!("Only in {}: {}", dir1.display(), rel.display());
            any_diff = true;
        } else {
            let had = diff_files(&p1, &p2, context, new_file)?;
            if had {
                any_diff = true;
            }
        }
    }

    Ok(if any_diff { 1 } else { 0 })
}

/// Compare two files and print a unified diff if they differ.
/// Returns `true` if there were differences.
fn diff_files(path1: &Path, path2: &Path, context: usize, treat_absent_as_empty: bool) -> Result<bool> {
    let read_or_empty = |path: &Path| -> Result<Vec<u8>> {
        if !path.exists() {
            if treat_absent_as_empty {
                return Ok(Vec::new());
            } else {
                return Err(anyhow::anyhow!(
                    "diff: {}: No such file or directory",
                    path.display()
                ));
            }
        }
        std::fs::read(path)
            .with_context(|| format!("diff: {}: cannot read file", path.display()))
    };

    let bytes1 = read_or_empty(path1)?;
    let bytes2 = read_or_empty(path2)?;

    let text1 = String::from_utf8_lossy(&bytes1);
    let text2 = String::from_utf8_lossy(&bytes2);

    let diff = TextDiff::from_lines(text1.as_ref(), text2.as_ref());

    // Check if there are any actual changes
    let has_changes = diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal);
    if !has_changes {
        return Ok(false);
    }

    // Print header
    let mtime1 = file_mtime_str(path1);
    let mtime2 = file_mtime_str(path2);
    println!("--- {}\t{}", path1.display(), mtime1);
    println!("+++ {}\t{}", path2.display(), mtime2);

    // Print hunks
    let mut unified_builder = diff.unified_diff();
    let unified = unified_builder.context_radius(context);
    for hunk in unified.iter_hunks() {
        // UnifiedHunkHeader implements Display as "@@ -L,S +L,S @@"
        println!("{}", hunk.header());
        for change in hunk.iter_changes() {
            let prefix = match change.tag() {
                ChangeTag::Delete => '-',
                ChangeTag::Insert => '+',
                ChangeTag::Equal => ' ',
            };
            let value = change.value();
            // Ensure each line ends with a newline
            if value.ends_with('\n') {
                print!("{}{}", prefix, value);
            } else {
                println!("{}{}", prefix, value);
                println!("\\ No newline at end of file");
            }
        }
    }

    Ok(true)
}

fn file_mtime_str(path: &Path) -> String {
    use std::time::UNIX_EPOCH;

    if !path.exists() {
        return String::from("1970-01-01 00:00:00.000000000 +0000");
    }

    match std::fs::metadata(path) {
        Ok(meta) => {
            match meta.modified() {
                Ok(mtime) => {
                    let duration = mtime
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default();
                    let secs = duration.as_secs();
                    let nanos = duration.subsec_nanos();

                    // Convert to date/time — simplified approach using time math
                    // Format: YYYY-MM-DD HH:MM:SS.NNNNNNNNN +0000
                    format_unix_timestamp(secs, nanos)
                }
                Err(_) => String::from("1970-01-01 00:00:00.000000000 +0000"),
            }
        }
        Err(_) => String::from("1970-01-01 00:00:00.000000000 +0000"),
    }
}

fn format_unix_timestamp(secs: u64, nanos: u32) -> String {
    // Days since epoch
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    // Convert days since epoch to Y/M/D using the Gregorian calendar algorithm
    let (year, month, day) = days_to_ymd(days_since_epoch as i64);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09} +0000",
        year, month, day, h, m, s, nanos
    )
}

fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}
