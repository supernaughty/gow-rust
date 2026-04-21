//! `uu_mkdir`: GNU `mkdir` — thin `std::fs::create_dir` / `create_dir_all` wrapper.
//!
//! Delegation strategy (CONTEXT.md D-27, RESEARCH.md Q5):
//! - Without `-p`: `std::fs::create_dir` — fails if the path exists or the parent
//!   is missing. Matches GNU semantics and POSIX.
//! - With `-p` / `--parents`: `std::fs::create_dir_all` — creates the full chain
//!   and is a no-op if the final path already exists as a directory. Rust std
//!   documents this explicitly, which means GOW issue #133 ("mkdir -p fails on
//!   existing dir") is resolved by the delegation itself — no custom loop needed.
//!
//! MSYS pre-convert (D-26): each path operand is run through
//! `gow_core::path::try_convert_msys_path` before the filesystem call so that
//! `/c/Users/newdir` becomes `C:\Users\newdir` on Windows.
//!
//! `-m MODE` is intentionally NOT supported in this plan (see PLAN.md Task 1
//! note): Windows has no POSIX mode bits, GNU allows the flag to be a no-op on
//! such platforms. Passing `-m` today produces the standard "unrecognized
//! option" error — deferred to a future plan that can wire ACLs if desired.

use std::ffi::OsString;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let parents = matches.get_flag("parents");
    let verbose = matches.get_flag("verbose");

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        eprintln!("mkdir: missing operand");
        return 1;
    }

    let mut exit_code = 0;
    for op in &operands {
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);

        // D-27 / RESEARCH.md Q5: `std::fs::create_dir_all` is POSIX-correct —
        // it returns Ok(()) when the path already exists AS A DIRECTORY. If it
        // exists as a file the call returns Err(AlreadyExists), which is also
        // the correct GNU behavior.
        let result = if parents {
            std::fs::create_dir_all(path)
        } else {
            std::fs::create_dir(path)
        };

        match result {
            Ok(()) => {
                if verbose {
                    println!("mkdir: created directory '{converted}'");
                }
            }
            Err(e) => {
                eprintln!("mkdir: cannot create directory '{converted}': {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}

fn uu_app() -> Command {
    Command::new("mkdir")
        .arg(
            Arg::new("parents")
                .short('p')
                .long("parents")
                .action(ArgAction::SetTrue)
                .help("no error if existing, make parent directories as needed"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("print a message for each created directory"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}
