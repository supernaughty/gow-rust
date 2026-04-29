use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "tac",
    about = "GNU tac — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Files to reverse (reads stdin if none provided)
    files: Vec<String>,
}

/// Reverse lines in a byte buffer and write to stdout.
/// Lines are split on `\n`. If the buffer ends with `\n`, the trailing empty
/// segment is dropped (i.e., `a\nb\nc\n` has 3 lines, not 4).
/// Each line is written followed by `\n` to normalise output.
fn reverse_and_write(buf: &[u8], out: &mut impl Write) {
    if buf.is_empty() {
        return;
    }

    // Split into lines (split on \n)
    let mut lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();

    // Drop trailing empty segment if buf ends with \n
    if buf.ends_with(b"\n") {
        lines.pop();
    }

    // Reverse in-place
    lines.reverse();

    // Write each line followed by \n
    for line in lines {
        let _ = out.write_all(line);
        let _ = out.write_all(b"\n");
    }
}

fn run(cli: &Cli) -> i32 {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        // Read all of stdin
        let mut buf = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buf) {
            eprintln!("tac: stdin: {e}");
            exit_code = 1;
        }
        reverse_and_write(&buf, &mut out);
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            match fs::read(&converted) {
                Ok(buf) => {
                    reverse_and_write(&buf, &mut out);
                }
                Err(e) => {
                    eprintln!("tac: {converted}: {e}");
                    exit_code = 1;
                }
            }
        }
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
            eprintln!("tac: {e}");
            return 2;
        }
    };
    run(&cli)
}
