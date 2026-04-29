//! `uu_whoami`: GNU whoami — Windows port. Prints the current username via GetUserNameW (U2-01).
//!
//! T-11-06-01: whoami reveals current username; standard Unix behavior, not a security issue.

use std::ffi::OsString;
use std::io::{self, Write};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;

#[derive(Parser, Debug)]
#[command(
    name = "whoami",
    about = "GNU whoami — Windows port. Prints the current username.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
}

/// Get current Windows username via GetUserNameW.
///
/// Returns `None` if the Win32 API call fails (should never happen for current user).
/// Buffer is UNLEN+1 = 257 characters to accommodate the longest possible Windows username.
fn get_current_username() -> Option<String> {
    let mut buf = [0u16; 257]; // UNLEN + 1 = 257
    let mut size = buf.len() as u32;
    // SAFETY: buf is a valid stack-allocated array of 257 u16 elements.
    // size is the buffer capacity in characters on input.
    // On success, size is updated to the number of characters written INCLUDING the null terminator.
    // Returns nonzero on success, 0 on failure.
    let ok = unsafe { GetUserNameW(buf.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return None;
    }
    // size includes null terminator on return — subtract 1 to exclude it
    let end = (size as usize).saturating_sub(1);
    Some(String::from_utf16_lossy(&buf[..end]))
}

fn run(_cli: &Cli) -> i32 {
    match get_current_username() {
        Some(name) => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            if let Err(e) = writeln!(out, "{}", name) {
                eprintln!("whoami: write error: {e}");
                return 1;
            }
            0
        }
        None => {
            eprintln!("whoami: cannot determine current user");
            1
        }
    }
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("whoami: {e}");
            return 2;
        }
    };
    run(&cli)
}
