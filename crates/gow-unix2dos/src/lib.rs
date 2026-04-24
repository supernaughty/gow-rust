//! `uu_unix2dos`: GNU `unix2dos` — convert LF line endings to CRLF in-place (CONV-02).
//!
//! Uses `gow_core::fs::atomic_rewrite` for Pitfall #4 / D-47 compliance
//! (same-dir tempfile + MoveFileExW REPLACE_EXISTING).
//!
//! Reuses the byte-level transforms from `uu_dos2unix` via path dependency.

use std::ffi::OsString;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

use gow_core::error::GowError;
use uu_dos2unix::transform::{lf_to_crlf, is_binary};

fn uu_app() -> Command {
    Command::new("unix2dos")
        .about("Convert LF line endings to CRLF in-place")
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Force conversion of binary files"),
        )
        .arg(
            Arg::new("keep-date")
                .short('k')
                .long("keepdate")
                .action(ArgAction::SetTrue)
                .help("Preserve the date of the input file"),
        )
        .arg(
            Arg::new("new-file")
                .short('n')
                .long("newfile")
                .num_args(2)
                .value_names(["INFILE", "OUTFILE"])
                .help("Write OUTFILE instead of modifying INFILE in-place"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress informational messages"),
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
    let force = matches.get_flag("force");
    let keep_date = matches.get_flag("keep-date");
    let quiet = matches.get_flag("quiet");

    // -n mode takes exactly two arguments — a src+dst pair
    if let Some(pair) = matches.get_many::<String>("new-file") {
        let names: Vec<&String> = pair.collect();
        if names.len() != 2 {
            eprintln!("unix2dos: -n requires two arguments: INFILE OUTFILE");
            return 1;
        }
        let src = gow_core::path::try_convert_msys_path(names[0]);
        let dst = gow_core::path::try_convert_msys_path(names[1]);
        return convert_to_new_file(Path::new(&src), Path::new(&dst), force, quiet);
    }

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("unix2dos: no operands given");
        eprintln!("usage: unix2dos [OPTIONS]... FILE...");
        return 1;
    }

    let mut exit_code = 0;
    for op in &operands {
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);
        match convert_in_place(path, force, keep_date, quiet) {
            Ok(_) => {} // success
            Err(e) => {
                eprintln!("unix2dos: {converted}: {e}");
                exit_code = 1;
            }
        }
    }
    exit_code
}

fn convert_in_place(
    path: &Path,
    force: bool,
    keep_date: bool,
    quiet: bool,
) -> std::io::Result<bool> {
    // Pre-read for binary detection
    let pre_bytes = std::fs::read(path)?;
    if !force && is_binary(&pre_bytes) {
        eprintln!("unix2dos: Skipping binary file {}", path.display());
        return Ok(false);
    }

    // Capture timestamps BEFORE rewrite if -k.
    let timestamps = if keep_date {
        let md = std::fs::metadata(path)?;
        Some((
            filetime::FileTime::from_last_access_time(&md),
            filetime::FileTime::from_last_modification_time(&md),
        ))
    } else {
        None
    };

    // Atomic rewrite via gow-core helper (D-47).
    gow_core::fs::atomic_rewrite(path, |bytes| Ok(lf_to_crlf(bytes))).map_err(|e| match e {
        GowError::Io { source, .. } => source,
        other => std::io::Error::other(other.to_string()),
    })?;

    if let Some((atime, mtime)) = timestamps {
        filetime::set_file_times(path, atime, mtime)?;
    }

    if !quiet {
        println!(
            "unix2dos: converting file {} to DOS format...",
            path.display()
        );
    }
    Ok(true)
}

fn convert_to_new_file(src: &Path, dst: &Path, force: bool, quiet: bool) -> i32 {
    let bytes = match std::fs::read(src) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("unix2dos: {}: {e}", src.display());
            return 1;
        }
    };
    if !force && is_binary(&bytes) {
        eprintln!("unix2dos: Skipping binary file {}", src.display());
        return 0;
    }
    let converted = lf_to_crlf(&bytes);
    if let Err(e) = std::fs::write(dst, &converted) {
        eprintln!("unix2dos: {}: {e}", dst.display());
        return 1;
    }
    if !quiet {
        println!(
            "unix2dos: converting file {} to file {} in DOS format...",
            src.display(),
            dst.display()
        );
    }
    0
}
