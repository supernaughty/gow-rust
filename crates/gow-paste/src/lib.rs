use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "paste",
    about = "GNU paste — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Delimiter list for joining columns (default: tab). Cycles through if multiple.
    #[arg(short = 'd', long = "delimiters", default_value = "\t")]
    delimiters: String,

    /// Serial mode: paste one file at a time rather than in parallel columns
    #[arg(short = 's', long = "serial", action = ArgAction::SetTrue)]
    serial: bool,

    /// Files to paste (use - for stdin). If none given, reads stdin.
    files: Vec<String>,
}

/// Parse escape sequences in the delimiter string: \t → tab, \n → newline, \\ → backslash.
fn parse_delimiters(s: &str) -> Vec<char> {
    let mut delimiters = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('t') => delimiters.push('\t'),
                Some('n') => delimiters.push('\n'),
                Some('\\') => delimiters.push('\\'),
                Some(other) => {
                    delimiters.push('\\');
                    delimiters.push(other);
                }
                None => delimiters.push('\\'),
            }
        } else {
            delimiters.push(c);
        }
    }
    if delimiters.is_empty() {
        delimiters.push('\t');
    }
    delimiters
}

/// Column source discriminant: stdin (reads from shared stdin iter) or a buffered file/empty.
enum ColSource {
    /// Reads from the shared stdin Lines iterator
    Stdin,
    /// Pre-built iterator (file or empty cursor)
    Buffered(Box<dyn BufRead>),
}

/// Read the next line from a column source.
/// `stdin_iter` is passed explicitly for Stdin variants.
fn next_line_from(
    source: &mut ColSource,
    stdin_iter: &mut dyn Iterator<Item = io::Result<String>>,
) -> Option<String> {
    match source {
        ColSource::Stdin => stdin_iter.next().and_then(|r| r.ok()),
        ColSource::Buffered(reader) => {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => None,
                Ok(_) => {
                    // Strip trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Some(line)
                }
                Err(_) => None,
            }
        }
    }
}

fn run(cli: &Cli) -> i32 {
    let delimiters = parse_delimiters(&cli.delimiters);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Build the operand list: if no files given, use single stdin "-"
    let operands: Vec<String> = if cli.files.is_empty() {
        vec!["-".to_string()]
    } else {
        cli.files.clone()
    };

    let n_cols = operands.len();
    let mut exit_code = 0i32;

    // Build column sources
    let mut sources: Vec<ColSource> = Vec::with_capacity(n_cols);

    for op in &operands {
        if op == "-" {
            sources.push(ColSource::Stdin);
        } else {
            let converted = gow_core::path::try_convert_msys_path(op);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => sources.push(ColSource::Buffered(Box::new(BufReader::new(f)))),
                Err(e) => {
                    eprintln!("paste: {converted}: {e}");
                    exit_code = 1;
                    // Push empty reader to maintain column alignment
                    sources.push(ColSource::Buffered(Box::new(BufReader::new(
                        Cursor::new(Vec::<u8>::new()),
                    ))));
                }
            }
        }
    }

    // Set up stdin iterator (used for all Stdin columns, shared via &mut dyn Iterator)
    let stdin_handle = io::stdin();
    let locked = stdin_handle.lock();
    let mut stdin_iter: Box<dyn Iterator<Item = io::Result<String>>> =
        Box::new(BufReader::new(locked).lines());

    if cli.serial {
        // Serial mode: for each source, read all lines and join with delimiters, then newline
        for source in &mut sources {
            let mut parts: Vec<String> = Vec::new();
            loop {
                match next_line_from(source, &mut *stdin_iter) {
                    Some(line) => parts.push(line),
                    None => break,
                }
            }
            if !parts.is_empty() {
                let joined = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if i + 1 < parts.len() {
                            let delim = delimiters[i % delimiters.len()];
                            format!("{}{}", p, delim)
                        } else {
                            p.clone()
                        }
                    })
                    .collect::<String>();
                let _ = writeln!(out, "{}", joined);
            }
        }
        return exit_code;
    }

    // Parallel mode: zip lines from all columns side-by-side
    loop {
        let mut row_parts: Vec<Option<String>> = Vec::with_capacity(n_cols);

        for source in &mut sources {
            let line = next_line_from(source, &mut *stdin_iter);
            row_parts.push(line);
        }

        // If ALL columns exhausted, stop
        if row_parts.iter().all(|p| p.is_none()) {
            break;
        }

        // Join columns with cycling delimiters
        for (i, part) in row_parts.iter().enumerate() {
            let text = part.as_deref().unwrap_or("");
            let _ = out.write_all(text.as_bytes());
            if i + 1 < n_cols {
                // Delimiter between column i and column i+1
                let delim_char = delimiters[i % delimiters.len()];
                let mut buf = [0u8; 4];
                let encoded = delim_char.encode_utf8(&mut buf);
                let _ = out.write_all(encoded.as_bytes());
            }
        }
        let _ = out.write_all(b"\n");
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
            eprintln!("paste: {e}");
            return 2;
        }
    };
    run(&cli)
}
