//! `uu_mv`: GNU mv — move (rename) files (FILE-04).
//!
//! Covers: FILE-04

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path};

use anyhow::{Context, Result};
use clap::Parser;
use filetime::{FileTime, set_file_times};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "mv", about = "Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.", version)]
struct Args {
    /// Do not prompt before overwriting
    #[arg(short, long, overrides_with = "no_clobber")]
    force: bool,

    /// Do not overwrite an existing file
    #[arg(short = 'n', long, overrides_with = "force")]
    no_clobber: bool,

    /// Explain what is being done
    #[arg(short, long)]
    verbose: bool,

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
            if e.use_stderr() {
                eprintln!("{}", e);
                return 1;
            } else {
                println!("{}", e);
                return 0;
            }
        }
    };

    match run(parsed) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("mv: {}", e);
            1
        }
    }
}

fn run(args: Args) -> Result<i32> {
    if args.paths.len() < 2 {
        eprintln!("mv: missing file operand");
        return Ok(1);
    }

    let (sources, dest_str) = args.paths.split_at(args.paths.len() - 1);
    let dest_path = Path::new(&dest_str[0]);
    let dest_is_dir = dest_path.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        eprintln!("mv: target '{}' is not a directory", dest_path.display());
        return Ok(1);
    }

    let mut exit_code = 0;
    for src_str in sources {
        let src_path = Path::new(src_str);
        if !src_path.exists() && !src_path.is_symlink() {
            eprintln!("mv: cannot stat '{}': No such file or directory", src_path.display());
            exit_code = 1;
            continue;
        }

        let target_dest = if dest_is_dir {
            match src_path.file_name() {
                Some(name) => dest_path.join(name),
                None => {
                    eprintln!("mv: invalid source path '{}'", src_path.display());
                    exit_code = 1;
                    continue;
                }
            }
        } else {
            dest_path.to_path_buf()
        };

        if let Err(e) = move_path(src_path, &target_dest, &args) {
            eprintln!("mv: {}", e);
            exit_code = 1;
        }
    }

    Ok(exit_code)
}

fn move_path(src: &Path, dest: &Path, args: &Args) -> Result<()> {
    // Canonicalize for same-file check to handle different path representations
    let src_canonical = src.canonicalize().ok();
    let dest_canonical = dest.canonicalize().ok();

    if let (Some(s), Some(d)) = (src_canonical, dest_canonical) {
        if s == d {
            return Err(anyhow::anyhow!("'{}' and '{}' are the same file", src.display(), dest.display()));
        }
    }

    if dest.exists() && args.no_clobber {
        if args.verbose {
            println!("skipping '{}', destination '{}' exists", src.display(), dest.display());
        }
        return Ok(());
    }

    if dest.exists() {
        if dest.is_dir() && !src.is_dir() {
            return Err(anyhow::anyhow!("cannot overwrite directory '{}' with non-directory '{}'", dest.display(), src.display()));
        }
        if !dest.is_dir() && src.is_dir() {
            return Err(anyhow::anyhow!("cannot overwrite non-directory '{}' with directory '{}'", dest.display(), src.display()));
        }
    }

    // Try same-volume rename first
    match fs::rename(src, dest) {
        Ok(_) => {
            if args.verbose {
                println!("renamed '{}' -> '{}'", src.display(), dest.display());
            }
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            // Pitfall 4: explicit cross-volume fallback
            cross_device_move(src, dest, args)
        }
        Err(e) => {
            // Re-attempting rename after manual removal of dest if it's a file
            if dest.exists() && !dest.is_dir() {
                 fs::remove_file(dest).ok();
                 match fs::rename(src, dest) {
                     Ok(_) => {
                        if args.verbose {
                            println!("renamed '{}' -> '{}'", src.display(), dest.display());
                        }
                        return Ok(());
                     }
                     Err(e2) if e2.kind() == io::ErrorKind::CrossesDevices => {
                        return cross_device_move(src, dest, args);
                     }
                     Err(e2) => return Err(anyhow::anyhow!("cannot move '{}' to '{}': {}", src.display(), dest.display(), e2)),
                 }
            }

            Err(anyhow::anyhow!("cannot move '{}' to '{}': {}", src.display(), dest.display(), e))
        }
    }
}

fn cross_device_move(src: &Path, dest: &Path, args: &Args) -> Result<()> {
    let metadata = fs::symlink_metadata(src).context("failed to get source metadata")?;
    
    if metadata.is_dir() {
        copy_and_remove_dir(src, dest, args)?;
    } else {
        copy_and_remove_file(src, dest, args, &metadata)?;
    }
    
    Ok(())
}

fn copy_and_remove_file(src: &Path, dest: &Path, args: &Args, src_metadata: &fs::Metadata) -> Result<()> {
    // If dest exists, we already checked no_clobber. std::fs::copy will overwrite.
    fs::copy(src, dest).map_err(|e| anyhow::anyhow!("cannot copy '{}' to '{}': {}", src.display(), dest.display(), e))?;
    
    // Preserve metadata
    preserve_metadata(src_metadata, dest)?;
    
    // Remove source
    fs::remove_file(src).map_err(|e| anyhow::anyhow!("cannot remove source '{}' after copy: {}", src.display(), e))?;
    
    if args.verbose {
        println!("moved '{}' -> '{}' (cross-device)", src.display(), dest.display());
    }
    
    Ok(())
}

fn copy_and_remove_dir(src: &Path, dest: &Path, args: &Args) -> Result<()> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    } else if !dest.is_dir() {
        return Err(anyhow::anyhow!("cannot overwrite non-directory '{}' with directory '{}'", dest.display(), src.display()));
    }

    // WalkDir for recursive copy
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(src)?;
        let target_path = dest.join(rel_path);

        let metadata = entry.metadata()?;
        
        if metadata.is_dir() {
            if !target_path.exists() {
                fs::create_dir(&target_path)?;
            }
            preserve_metadata(&metadata, &target_path)?;
        } else if metadata.file_type().is_symlink() {
            let target = fs::read_link(entry.path())?;
            if target_path.exists() {
                if target_path.is_dir() {
                    fs::remove_dir_all(&target_path)?;
                } else {
                    fs::remove_file(&target_path)?;
                }
            }
            // Use gow_core for link creation if possible, but mv doesn't have -s.
            // Here we are copying a symlink.
            #[cfg(target_os = "windows")]
            {
                 if metadata.is_dir() {
                     std::os::windows::fs::symlink_dir(&target, &target_path)?;
                 } else {
                     std::os::windows::fs::symlink_file(&target, &target_path)?;
                 }
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::os::unix::fs::symlink(&target, &target_path)?;
            }
        } else {
            // Regular file
            fs::copy(entry.path(), &target_path)?;
            preserve_metadata(&metadata, &target_path)?;
        }
    }

    // After successful copy, remove source directory
    fs::remove_dir_all(src).map_err(|e| anyhow::anyhow!("cannot remove source directory '{}' after copy: {}", src.display(), e))?;

    if args.verbose {
        println!("moved directory '{}' -> '{}' (cross-device)", src.display(), dest.display());
    }

    Ok(())
}

fn preserve_metadata(src_metadata: &fs::Metadata, dest: &Path) -> Result<()> {
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
