//! `uu_touch`: GNU `touch` — update or create file timestamps.
//!
//! Full flag set per D-19c: `-a`, `-m`, `-c`, `-r`, `-d`, `-t`, `-h`.
//!
//! Reference: RESEARCH.md Q1 (parse_datetime), Q2 (filetime symlink-self),
//! CONTEXT.md D-19/D-29.

mod date;
mod timestamps;

use std::ffi::OsString;
use std::path::Path;

use clap::{Arg, ArgAction, Command};
use filetime::FileTime;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    let change_atime = matches.get_flag("access");
    let change_mtime = matches.get_flag("modify");
    let no_create = matches.get_flag("no-create");
    let no_deref = matches.get_flag("no-dereference");
    let ref_file: Option<String> = matches.get_one::<String>("reference").cloned();
    let date_str: Option<String> = matches.get_one::<String>("date").cloned();
    let stamp_str: Option<String> = matches.get_one::<String>("stamp").cloned();

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("touch: missing file operand");
        return 1;
    }

    // If neither -a nor -m was specified, update BOTH (GNU default per D-29).
    let (update_atime, update_mtime) = if !change_atime && !change_mtime {
        (true, true)
    } else {
        (change_atime, change_mtime)
    };

    // Resolve the time source once (reference file, -d string, -t stamp, or now).
    let (source_atime, source_mtime) = match resolve_times(
        date_str.as_deref(),
        stamp_str.as_deref(),
        ref_file.as_deref(),
    ) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("touch: {e}");
            return 1;
        }
    };

    let mut exit_code = 0;
    for op in &operands {
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);

        // Does the entry exist (as a file/dir or as a symlink)?
        let exists = path.exists() || path.is_symlink();

        if !exists {
            if no_create {
                // -c / --no-create: do not create, do not error (GNU behavior).
                continue;
            }
            if let Err(e) = std::fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(path)
            {
                eprintln!("touch: cannot create '{converted}': {e}");
                exit_code = 1;
                continue;
            }
        }

        // Determine final atime/mtime: if we're only updating one of the two,
        // preserve the other from the file's current metadata.
        let (final_atime, final_mtime) = match (update_atime, update_mtime) {
            (true, true) => (source_atime, source_mtime),
            (true, false) => {
                let existing_mtime = current_mtime(path, no_deref).unwrap_or(source_mtime);
                (source_atime, existing_mtime)
            }
            (false, true) => {
                let existing_atime = current_atime(path, no_deref).unwrap_or(source_atime);
                (existing_atime, source_mtime)
            }
            (false, false) => unreachable!("both flags false is rewritten to (true, true) above"),
        };

        if let Err(e) = timestamps::apply(path, final_atime, final_mtime, no_deref) {
            eprintln!("touch: setting times of '{converted}': {e}");
            exit_code = 1;
        }
    }

    exit_code
}

fn resolve_times(
    date_str: Option<&str>,
    stamp_str: Option<&str>,
    ref_file: Option<&str>,
) -> Result<(FileTime, FileTime), date::TouchError> {
    // Priority: -r > -d > -t > now. (GNU's own tie-breaking also favors -r first.)
    if let Some(r) = ref_file {
        let md = std::fs::metadata(r).map_err(|e| date::TouchError::Io {
            path: r.to_string(),
            source: e,
        })?;
        let atime = FileTime::from_last_access_time(&md);
        let mtime = FileTime::from_last_modification_time(&md);
        return Ok((atime, mtime));
    }
    if let Some(d) = date_str {
        let t = date::parse_touch_date(d, jiff::Zoned::now())?;
        return Ok((t, t));
    }
    if let Some(s) = stamp_str {
        let t = date::parse_touch_stamp(s)?;
        return Ok((t, t));
    }
    let now = FileTime::now();
    Ok((now, now))
}

fn current_atime(path: &Path, no_deref: bool) -> Option<FileTime> {
    let md = if no_deref {
        std::fs::symlink_metadata(path).ok()?
    } else {
        std::fs::metadata(path).ok()?
    };
    Some(FileTime::from_last_access_time(&md))
}

fn current_mtime(path: &Path, no_deref: bool) -> Option<FileTime> {
    let md = if no_deref {
        std::fs::symlink_metadata(path).ok()?
    } else {
        std::fs::metadata(path).ok()?
    };
    Some(FileTime::from_last_modification_time(&md))
}

fn uu_app() -> Command {
    // `touch` uses `-h` for `--no-dereference`, matching GNU coreutils. We
    // therefore disable clap's auto-generated short `-h` for help; `--help`
    // remains available via clap's long flag.
    Command::new("touch")
        .disable_help_flag(true)
        .arg(
            clap::Arg::new("help")
                .long("help")
                .action(ArgAction::Help)
                .help("print help and exit"),
        )
        .arg(
            Arg::new("access")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("change only the access time"),
        )
        .arg(
            Arg::new("modify")
                .short('m')
                .action(ArgAction::SetTrue)
                .help("change only the modification time"),
        )
        .arg(
            Arg::new("no-create")
                .short('c')
                .long("no-create")
                .action(ArgAction::SetTrue)
                .help("do not create any files"),
        )
        .arg(
            Arg::new("reference")
                .short('r')
                .long("reference")
                .num_args(1)
                .help("use FILE's timestamps instead of current time"),
        )
        .arg(
            Arg::new("date")
                .short('d')
                .long("date")
                .num_args(1)
                .help("parse STRING and use it as the time"),
        )
        .arg(
            Arg::new("stamp")
                .short('t')
                .num_args(1)
                .help("use [[CC]YY]MMDDhhmm[.ss] format instead of current time"),
        )
        .arg(
            Arg::new("no-dereference")
                .short('h')
                .long("no-dereference")
                .action(ArgAction::SetTrue)
                .help("affect each symlink instead of any referenced file"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

#[cfg(test)]
mod tests {
    use super::uu_app;

    #[test]
    fn uu_app_builds_without_panic() {
        let _ = uu_app();
    }
}
