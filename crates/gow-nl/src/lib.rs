use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "nl",
    about = "GNU nl — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Body numbering: a (all lines), t (non-empty lines, default), n (no lines)
    #[arg(short = 'b', long = "body-numbering", default_value = "t")]
    body_numbering: String,

    /// Line number field width (default 6)
    #[arg(short = 'w', long = "number-width", default_value = "6")]
    width: usize,

    /// Separator between line number and line text (default tab)
    #[arg(short = 's', long = "number-separator", default_value = "\t")]
    separator: String,

    /// Starting line number (default 1)
    #[arg(short = 'v', long = "starting-line-number", default_value = "1")]
    start: i64,

    /// Increment between line numbers (default 1)
    #[arg(short = 'i', long = "line-increment", default_value = "1")]
    increment: i64,

    /// Input files (reads stdin if none given)
    files: Vec<String>,
}

fn process<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    body_numbering: &str,
    width: usize,
    separator: &str,
    start: i64,
    increment: i64,
) -> io::Result<()> {
    let mut line_num = start;
    for line_result in reader.lines() {
        let line = line_result?;
        match body_numbering {
            "n" => {
                // GNU nl -b n: no number, no separator — raw line only
                writeln!(writer, "{}", line)?;
            }
            "t" => {
                if line.is_empty() {
                    // GNU nl -b t: blank line emitted as bare newline with no number/separator
                    writeln!(writer)?;
                } else {
                    write!(writer, "{:>width$}{}{}\n", line_num, separator, line, width = width)?;
                    line_num += increment;
                }
            }
            "a" => {
                // Number all lines including blank
                write!(writer, "{:>width$}{}{}\n", line_num, separator, line, width = width)?;
                line_num += increment;
            }
            _ => {
                // Should not reach here — validated before calling process
                write!(writer, "{}{}\n", separator, line)?;
            }
        }
    }
    Ok(())
}

fn run(cli: &Cli) -> i32 {
    // Validate body_numbering
    if !matches!(cli.body_numbering.as_str(), "a" | "t" | "n") {
        eprintln!("nl: invalid body numbering style: '{}'", cli.body_numbering);
        return 1;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        if let Err(e) = process(
            reader,
            &mut out,
            &cli.body_numbering,
            cli.width,
            &cli.separator,
            cli.start,
            cli.increment,
        ) {
            eprintln!("nl: stdin: {e}");
            exit_code = 1;
        }
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            match File::open(Path::new(&converted)) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    if let Err(e) = process(
                        reader,
                        &mut out,
                        &cli.body_numbering,
                        cli.width,
                        &cli.separator,
                        cli.start,
                        cli.increment,
                    ) {
                        eprintln!("nl: {converted}: {e}");
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("nl: {converted}: {e}");
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
            eprintln!("nl: {e}");
            return 2;
        }
    };
    run(&cli)
}
