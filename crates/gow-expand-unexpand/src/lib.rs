use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "expand",
    about = "GNU expand/unexpand — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Tab stop width (default 8)
    #[arg(short = 't', long = "tabs", default_value = "8")]
    tabs: usize,

    /// (unexpand only) Convert all blanks, not just leading whitespace
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
    all: bool,

    /// Input files (reads stdin if none given)
    files: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Expand,
    Unexpand,
}

fn detect_mode(invoked_as: &str) -> Mode {
    if invoked_as == "unexpand" {
        Mode::Unexpand
    } else {
        Mode::Expand
    }
}

/// Expand tabs to spaces in a single line (already stripped of trailing newline).
/// Returns the expanded string.
fn expand_line(line: &str, tab_width: usize) -> String {
    let mut out = String::with_capacity(line.len() + 16);
    let mut col = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            // Pad to next multiple of tab_width
            let pad = tab_width - (col % tab_width);
            for _ in 0..pad {
                out.push(' ');
            }
            col += pad;
        } else {
            out.push(ch);
            col += 1;
        }
    }
    out
}

/// Convert leading (and optionally all) spaces to tabs in a single line.
/// Column positions are tracked from the start of the line.
fn unexpand_line(line: &str, tab_width: usize, all_blanks: bool) -> String {
    let mut out = String::with_capacity(line.len());
    let mut col = 0usize;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_leading = true;

    while i < chars.len() {
        if chars[i] == ' ' && (in_leading || all_blanks) {
            // Count consecutive spaces in this run
            let run_start = i;
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            let count = i - run_start;

            // Convert this run of spaces to tabs+spaces using column-aware arithmetic
            let mut remaining = count;
            while remaining > 0 {
                let to_next = tab_width - (col % tab_width);
                if remaining >= to_next {
                    // Enough spaces to reach next tab stop — emit a tab
                    out.push('\t');
                    col += to_next;
                    remaining -= to_next;
                } else {
                    // Not enough to reach next tab stop — emit remaining as spaces
                    for _ in 0..remaining {
                        out.push(' ');
                    }
                    col += remaining;
                    remaining = 0;
                }
            }
            continue;
        }

        // Non-space character
        let c = chars[i];
        if c != ' ' && c != '\t' {
            in_leading = false;
        }
        if c == '\t' {
            // A literal tab in the input — advance column to next tab stop
            let to_next = tab_width - (col % tab_width);
            col += to_next;
        } else {
            col += 1;
        }
        out.push(c);
        i += 1;
    }

    out
}

fn process<R: BufRead, W: Write>(
    mode: Mode,
    cli: &Cli,
    reader: R,
    mut writer: W,
) -> io::Result<()> {
    for line_result in reader.lines() {
        let line = line_result?;
        let transformed = match mode {
            Mode::Expand => expand_line(&line, cli.tabs),
            Mode::Unexpand => unexpand_line(&line, cli.tabs, cli.all),
        };
        writer.write_all(transformed.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn run(cli: &Cli, invoked_as: &str) -> i32 {
    let mode = detect_mode(invoked_as);

    if cli.tabs == 0 {
        eprintln!("{}: tab size must be positive", invoked_as);
        return 1;
    }

    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let reader = BufReader::new(stdin.lock());
        if let Err(e) = process(mode, cli, reader, stdout.lock()) {
            eprintln!("{}: stdin: {e}", invoked_as);
            return 1;
        }
        return 0;
    }

    for file_path in &cli.files {
        let converted = gow_core::path::try_convert_msys_path(file_path);
        match File::open(Path::new(&converted)) {
            Ok(file) => {
                let stdout = io::stdout();
                let reader = BufReader::new(file);
                if let Err(e) = process(mode, cli, reader, stdout.lock()) {
                    eprintln!("{}: {converted}: {e}", invoked_as);
                    exit_code = 1;
                }
            }
            Err(e) => {
                eprintln!("{}: {converted}: {e}", invoked_as);
                exit_code = 1;
            }
        }
    }

    exit_code
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();

    // Detect invocation name for argv[0] mode switching
    let invoked_as = args_vec
        .first()
        .map(|s| {
            Path::new(s)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        })
        .unwrap_or_default()
        .to_string();

    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {e}", if invoked_as == "unexpand" { "unexpand" } else { "expand" });
            return 2;
        }
    };

    run(&cli, &invoked_as)
}
