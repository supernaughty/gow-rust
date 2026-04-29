//! `uu_df`: GNU df — disk free space using GetLogicalDriveStringsW + GetDiskFreeSpaceExW (U-09).
//!
//! T-10-05-02: Unsafe Win32 calls use only local buffer pointers — minimal attack surface.
//! T-10-05-03: Drives returning FALSE (empty CD-ROMs, disconnected network) are silently skipped.

use std::ffi::OsString;
use std::io::Write;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};

#[derive(Parser, Debug)]
#[command(
    name = "df",
    about = "GNU df — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Print sizes in human-readable format (e.g. 1K, 234M, 2G)
    #[arg(short = 'h', long = "human-readable", action = ArgAction::SetTrue)]
    human: bool,
}

/// Enumerate all logical drive roots via GetLogicalDriveStringsW.
/// Returns strings like ["C:\\", "D:\\", ...]
fn get_drives() -> Vec<String> {
    let mut buf = [0u16; 256];
    // SAFETY: buf and len are valid; returns total character count written, or 0 on error
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 {
        return Vec::new();
    }
    let mut drives = Vec::new();
    let mut start = 0usize;
    for i in 0..len as usize {
        if buf[i] == 0 {
            if i > start {
                drives.push(String::from_utf16_lossy(&buf[start..i]));
            }
            start = i + 1;
        }
    }
    drives
}

/// Query free space for a drive root. Returns `Some((free_available, total_bytes, total_free))`.
/// Returns `None` if the drive does not respond (e.g. empty CD-ROM, disconnected network drive).
/// Pitfall 4: FALSE return MUST be silently skipped — do not propagate as error.
fn get_disk_free(root: &str) -> Option<(u64, u64, u64)> {
    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let mut free_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;
    // SAFETY: wide is null-terminated with trailing 0, all output pointers are valid stack references
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_available,
            &mut total_bytes,
            &mut total_free,
        )
    };
    if ok != 0 {
        Some((free_available, total_bytes, total_free))
    } else {
        None
    }
}

/// Human-readable formatting (GNU binary SI units: 1K = 1024).
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

/// Format size as 1K-blocks (default) or human-readable (-h).
fn format_size(bytes: u64, human: bool) -> String {
    if human {
        human_readable(bytes)
    } else {
        format!("{}", (bytes + 1023) / 1024)
    }
}

/// Compute Use% column: round-half-up percentage of used bytes.
fn percent_used(total: u64, free_available: u64) -> String {
    if total == 0 {
        return "-".to_string();
    }
    let used = total.saturating_sub(free_available);
    // Round half-up: (used * 100 + total/2) / total
    let pct = (used as f64 / total as f64 * 100.0).round() as u64;
    format!("{}%", pct)
}

fn run(cli: &Cli) -> i32 {
    let drives = get_drives();
    if drives.is_empty() {
        eprintln!("df: no drives detected");
        return 1;
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // Print header matching GNU df format
    if cli.human {
        let _ = writeln!(out, "Filesystem      Size  Used Avail Use% Mounted on");
    } else {
        let _ = writeln!(
            out,
            "Filesystem     1K-blocks      Used Available Use% Mounted on"
        );
    }

    for drive in &drives {
        let info = get_disk_free(drive);
        let Some((free_available, total_bytes, _total_free)) = info else {
            // T-10-05-03: silently skip drives that return FALSE
            continue;
        };
        let used = total_bytes.saturating_sub(free_available);
        let size_col = format_size(total_bytes, cli.human);
        let used_col = format_size(used, cli.human);
        let avail_col = format_size(free_available, cli.human);
        let pct_col = percent_used(total_bytes, free_available);

        // Right-align columns to mirror GNU df layout
        if cli.human {
            let _ = writeln!(
                out,
                "{:<14} {:>5} {:>5} {:>5} {:>4} {}",
                drive, size_col, used_col, avail_col, pct_col, drive
            );
        } else {
            let _ = writeln!(
                out,
                "{:<14} {:>9} {:>9} {:>9} {:>4} {}",
                drive, size_col, used_col, avail_col, pct_col, drive
            );
        }
    }

    0
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("df: {e}");
            return 2;
        }
    };
    run(&cli)
}
