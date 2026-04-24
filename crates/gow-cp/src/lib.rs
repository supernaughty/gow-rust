//! `uu_cp`: GNU cp — copy files and directories with -r/-p preservation (FILE-03).
//!
//! Covers: FILE-03

use std::ffi::OsString;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use filetime::{FileTime, set_file_times};
use walkdir::WalkDir;

use gow_core::fs::{create_link, LinkKind};

#[derive(Parser, Debug)]
#[command(name = "cp", about = "Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY.", version)]
struct Args {
    /// Copy directories recursively
    #[arg(short, short_alias = 'R', long, alias = "recursive")]
    recursive: bool,

    /// If an existing destination file cannot be opened, remove it and try again
    #[arg(short, long)]
    force: bool,

    /// Preserve the specified attributes (default: mode,ownership,timestamps)
    #[arg(short, long)]
    preserve: bool,

    /// Never follow symbolic links in SOURCE
    #[arg(short = 'P', long = "no-dereference", conflicts_with_all = ["dereference", "follow_command_line"])]
    no_dereference: bool,

    /// Always follow symbolic links in SOURCE
    #[arg(short = 'L', long = "dereference", conflicts_with_all = ["no_dereference", "follow_command_line"])]
    dereference: bool,

    /// Follow command-line symbolic links in SOURCE
    #[arg(short = 'H', conflicts_with_all = ["no_dereference", "dereference"])]
    follow_command_line: bool,

    /// Explained what is being done
    #[arg(short, long)]
    verbose: bool,

    /// Same as -dR --preserve=all
    #[arg(short, long)]
    archive: bool,

    /// Target sources and destination
    #[arg(required = true, value_name = "PATH")]
    paths: Vec<String>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args: Vec<OsString> = args.into_iter().collect();
    let args_strings: Vec<String> = args.iter().map(|s| s.to_string_lossy().into_owned()).collect();
    let normalized = gow_core::path::normalize_file_args(&args_strings);

    let parsed = match Args::try_parse_from(normalized) {
        Ok(a) => a,
        Err(e) => {
            // Remap exit code 2 -> 1 for GNU parity (D-02)
            if e.use_stderr() {
                eprintln!("{}", e);
                return 1;
            } else {
                println!("{}", e);
                return 0;
            }
        }
    };

    run(parsed)
}

fn run(args: Args) -> i32 {
    let mut recursive = args.recursive;
    let mut preserve = args.preserve;
    let mut no_dereference = args.no_dereference;

    if args.archive {
        recursive = true;
        preserve = true;
        no_dereference = true;
    }

    let mut no_dereference_cmd = no_dereference;
    let mut no_dereference_traversal = no_dereference;

    if recursive && !args.dereference && !args.follow_command_line {
        no_dereference_cmd = true;
        no_dereference_traversal = true;
    }
    
    if args.follow_command_line {
        no_dereference_cmd = false;
        no_dereference_traversal = true;
    }

    if args.paths.len() < 2 {
        eprintln!("cp: missing destination file operand after '{}'", args.paths[0]);
        return 1;
    }

    let (sources, dest_str) = args.paths.split_at(args.paths.len() - 1);
    let dest_path = Path::new(&dest_str[0]);

    let dest_is_dir = dest_path.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        eprintln!("cp: target '{}' is not a directory", dest_path.display());
        return 1;
    }

    let mut exit_code = 0;
    for src_str in sources {
        let src_path = Path::new(src_str);
        
        if !src_path.exists() && !src_path.is_symlink() {
            eprintln!("cp: cannot stat '{}': No such file or directory", src_path.display());
            exit_code = 1;
            continue;
        }

        let target_dest = if dest_is_dir {
            match src_path.file_name() {
                Some(name) => dest_path.join(name),
                None => {
                    eprintln!("cp: invalid source path '{}'", src_path.display());
                    exit_code = 1;
                    continue;
                }
            }
        } else {
            dest_path.to_path_buf()
        };

        if src_path.is_dir() {
            if !recursive {
                eprintln!("cp: -r not specified; omitting directory '{}'", src_path.display());
                continue;
            }
            if let Err(e) = copy_dir(src_path, &target_dest, &args, recursive, preserve, no_dereference_cmd, no_dereference_traversal) {
                eprintln!("cp: {}", e);
                exit_code = 1;
            }
        } else {
            if let Err(e) = copy_file(src_path, &target_dest, &args, preserve, no_dereference_cmd) {
                eprintln!("cp: {}", e);
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn copy_file(src: &Path, dest: &Path, args: &Args, preserve: bool, no_dereference: bool) -> Result<()> {
    if src == dest {
        return Err(anyhow::anyhow!("'{}' and '{}' are the same file", src.display(), dest.display()));
    }

    let src_metadata = if no_dereference {
        fs::symlink_metadata(src)?
    } else {
        fs::metadata(src)?
    };

    if src_metadata.file_type().is_symlink() {
        // Copy the symlink itself
        if dest.exists() {
            if args.force {
                if dest.is_dir() {
                    fs::remove_dir_all(dest)?;
                } else {
                    fs::remove_file(dest)?;
                }
            } else {
                // GNU cp behavior: fail if dest exists and not force?
                // Actually it might just overwrite if it's a file.
            }
        }
        
        let target = fs::read_link(src)?;
        match create_link(&target, dest, LinkKind::Symbolic) {
            Ok(gow_core::fs::LinkOutcome::Junction) => {
                eprintln!("cp: warning: created junction instead of symlink for '{}'", dest.display());
            }
            Ok(_) => {}
            Err(e) => return Err(anyhow::anyhow!("cannot create symlink '{}': {}", dest.display(), e)),
        }
        
        if args.verbose {
            println!("'{}' -> '{}'", src.display(), dest.display());
        }
        return Ok(());
    }

    if dest.exists() && args.force {
        // Try to open it for writing. If it fails, remove it.
        if fs::OpenOptions::new().write(true).open(dest).is_err() {
            fs::remove_file(dest)?;
        }
    }

    fs::copy(src, dest).map_err(|e| anyhow::anyhow!("cannot copy '{}' to '{}': {}", src.display(), dest.display(), e))?;

    if preserve {
        preserve_metadata(src, dest, &src_metadata)?;
    }

    if args.verbose {
        println!("'{}' -> '{}'", src.display(), dest.display());
    }

    Ok(())
}

fn copy_dir(src: &Path, dest: &Path, args: &Args, _recursive: bool, preserve: bool, no_dereference_cmd: bool, no_dereference_traversal: bool) -> Result<()> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    } else if !dest.is_dir() {
        return Err(anyhow::anyhow!("cannot overwrite non-directory '{}' with directory '{}'", dest.display(), src.display()));
    }

    let mut walk = WalkDir::new(src);
    if !no_dereference_traversal {
        walk = walk.follow_links(true);
    }

    for entry in walk {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("cp: {}", e);
                continue;
            }
        };
        let rel_path = entry.path().strip_prefix(src)?;
        let target_path = dest.join(rel_path);

        let is_root = rel_path.as_os_str().is_empty();
        
        let metadata = if is_root {
            if no_dereference_cmd {
                entry.path().symlink_metadata()?
            } else {
                entry.path().metadata()?
            }
        } else {
            // entry.metadata() or entry.path().metadata()?
            // WalkDir's entry.metadata() respects follow_links.
            entry.metadata().map_err(|e| anyhow::anyhow!("cannot stat '{}': {}", entry.path().display(), e))?
        };

        if is_root {
            if preserve {
                preserve_metadata(entry.path(), &target_path, &metadata)?;
            }
            continue;
        }

        if metadata.is_dir() {
            if !target_path.exists() {
                fs::create_dir(&target_path)?;
            }
            if preserve {
                preserve_metadata(entry.path(), &target_path, &metadata)?;
            }
        } else if metadata.file_type().is_symlink() {
            let target = fs::read_link(entry.path())?;
            if target_path.exists() && args.force {
                if target_path.is_dir() {
                    fs::remove_dir_all(&target_path)?;
                } else {
                    fs::remove_file(&target_path)?;
                }
            }
            match create_link(&target, &target_path, LinkKind::Symbolic) {
                Ok(gow_core::fs::LinkOutcome::Junction) => {
                    eprintln!("cp: warning: created junction instead of symlink for '{}'", target_path.display());
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("cp: cannot create symlink '{}': {}", target_path.display(), e);
                }
            }
            if args.verbose {
                println!("'{}' -> '{}'", entry.path().display(), target_path.display());
            }
        } else {
            // File
            if target_path.exists() && args.force && fs::OpenOptions::new().write(true).open(&target_path).is_err() {
                fs::remove_file(&target_path)?;
            }
            fs::copy(entry.path(), &target_path)?;
            if preserve {
                preserve_metadata(entry.path(), &target_path, &metadata)?;
            }
            if args.verbose {
                println!("'{}' -> '{}'", entry.path().display(), target_path.display());
            }
        }
    }

    Ok(())
}

fn preserve_metadata(_src: &Path, dest: &Path, src_metadata: &fs::Metadata) -> Result<()> {
    // Preserve timestamps
    let atime = FileTime::from_last_access_time(src_metadata);
    let mtime = FileTime::from_last_modification_time(src_metadata);
    set_file_times(dest, atime, mtime).context("failed to preserve timestamps")?;

    // Preserve read-only bit
    let mut perms = dest.metadata()?.permissions();
    perms.set_readonly(src_metadata.permissions().readonly());
    fs::set_permissions(dest, perms).context("failed to preserve permissions")?;

    Ok(())
}
