//! `uu_rm`: GNU rm — remove files or directories.
//!
//! Covers: FILE-05, FOUND-07 (recursive), D-42 (preserve-root), D-45 (read-only)

use anyhow::Result;
use clap::Parser;
use gow_core::fs::{clear_readonly, is_drive_root};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "rm", about = "remove files or directories", disable_help_flag = true)]
struct Args {
    /// remove directories and their contents recursively
    #[arg(short, short_alias = 'R', long)]
    recursive: bool,

    /// ignore nonexistent files and arguments, never prompt
    #[arg(short, long)]
    force: bool,

    /// prompt before every removal
    #[arg(short)]
    interactive: bool,

    /// explain what is being done
    #[arg(short, long)]
    verbose: bool,

    /// do not remove '/' (default)
    #[arg(long, overrides_with = "no_preserve_root")]
    preserve_root: bool,

    /// do not treat '/' specially
    #[arg(long, overrides_with = "preserve_root")]
    no_preserve_root: bool,

    /// files to remove
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    #[arg(short, long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}

impl Args {
    fn preserve_root(&self) -> bool {
        !self.no_preserve_root
    }
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return 2;
        }
    };

    if args.files.is_empty() {
        if !args.force {
            eprintln!("rm: missing operand");
            return 2;
        }
        return 0;
    }

    let mut exit_code = 0;
    let preserve_root = args.preserve_root();

    for file in &args.files {
        let path = Path::new(file);

        if preserve_root && is_drive_root(path) {
            eprintln!("rm: it is dangerous to operate recursively on '{}'", file);
            eprintln!("rm: use --no-preserve-root to override this failsafe");
            exit_code = 1;
            continue;
        }

        if !path.exists() && !path.is_symlink() {
            if !args.force {
                eprintln!("rm: cannot remove '{}': No such file or directory", file);
                exit_code = 1;
            }
            continue;
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => {
                if !args.force {
                    eprintln!("rm: cannot remove '{}': {}", file, e);
                    exit_code = 1;
                }
                continue;
            }
        };

        if metadata.is_dir() {
            if !args.recursive {
                eprintln!("rm: cannot remove '{}': Is a directory", file);
                exit_code = 1;
                continue;
            }

            if let Err(e) = remove_recursive(path, &args) {
                eprintln!("rm: {}", e);
                exit_code = 1;
            }
        } else {
            if let Err(e) = remove_file(path, &args) {
                eprintln!("rm: {}", e);
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn remove_file(path: &Path, args: &Args) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    let is_readonly = metadata.permissions().readonly();

    if args.interactive {
        let msg = if is_readonly {
            format!("remove write-protected regular file '{}'?", path.display())
        } else {
            format!("remove regular file '{}'?", path.display())
        };
        if !prompt(&msg)? {
            return Ok(());
        }
    } else if !args.force && is_readonly && !prompt(&format!(
        "remove write-protected regular file '{}'?",
        path.display()
    ))? {
        return Ok(());
    }

    if args.force || is_readonly {
        let _ = clear_readonly(path);
    }

    fs::remove_file(path).map_err(|e| {
        anyhow::anyhow!("cannot remove '{}': {}", path.display(), e)
    })?;

    if args.verbose {
        println!("removed '{}'", path.display());
    }

    Ok(())
}

fn remove_recursive(path: &Path, args: &Args) -> Result<()> {
    // contents_first(true) is essential for removing directories:
    // we must remove the children before the parent.
    for entry in WalkDir::new(path).contents_first(true).min_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if !args.force {
                    eprintln!("rm: cannot access '{}': {}", path.display(), e);
                }
                continue;
            }
        };

        let entry_path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            let is_readonly = metadata.permissions().readonly();
            if args.interactive {
                let msg = if is_readonly {
                    format!("remove write-protected directory '{}'?", entry_path.display())
                } else {
                    format!("remove directory '{}'?", entry_path.display())
                };
                if !prompt(&msg)? {
                    continue;
                }
            }
            if args.force || is_readonly {
                let _ = clear_readonly(entry_path);
            }
            fs::remove_dir(entry_path).map_err(|e| {
                anyhow::anyhow!("cannot remove directory '{}': {}", entry_path.display(), e)
            })?;
            if args.verbose {
                println!("removed directory '{}'", entry_path.display());
            }
        } else {
            remove_file(entry_path, args)?;
        }
    }

    // Finally remove the top-level directory itself
    let metadata = fs::symlink_metadata(path)?;
    let is_readonly = metadata.permissions().readonly();
    if args.interactive {
        let msg = if is_readonly {
            format!("remove write-protected directory '{}'?", path.display())
        } else {
            format!("remove directory '{}'?", path.display())
        };
        if !prompt(&msg)? {
            return Ok(());
        }
    }
    if args.force || is_readonly {
        let _ = clear_readonly(path);
    }
    fs::remove_dir(path).map_err(|e| {
        anyhow::anyhow!("cannot remove directory '{}': {}", path.display(), e)
    })?;
    if args.verbose {
        println!("removed directory '{}'", path.display());
    }

    Ok(())
}

fn prompt(message: &str) -> Result<bool> {
    print!("rm: {} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input.starts_with('y'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parse() {
        let args = vec![OsString::from("rm"), OsString::from("-rf"), OsString::from("foo")];
        let parsed = Args::try_parse_from(args).unwrap();
        assert!(parsed.recursive);
        assert!(parsed.force);
        assert!(parsed.preserve_root());
        assert_eq!(parsed.files, vec!["foo"]);
    }

    #[test]
    fn test_args_no_preserve_root() {
        let args = vec![OsString::from("rm"), OsString::from("--no-preserve-root"), OsString::from("foo")];
        let parsed = Args::try_parse_from(args).unwrap();
        assert!(!parsed.preserve_root());
    }
}
