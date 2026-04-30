use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "fmt",
    about = "GNU fmt — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Maximum line width (default 75)
    #[arg(short = 'w', long = "width", default_value = "75")]
    width: usize,

    /// Files to format (reads stdin if none provided)
    files: Vec<String>,
}

/// Flush accumulated paragraph words as word-wrapped output at `width` columns.
fn flush_paragraph(para_words: &mut Vec<String>, width: usize, out: &mut dyn Write) {
    if para_words.is_empty() {
        return;
    }
    let mut line_len = 0usize;
    let mut first_on_line = true;
    for word in para_words.iter() {
        let word_len = word.len();
        if !first_on_line && line_len + 1 + word_len > width {
            let _ = out.write_all(b"\n");
            line_len = 0;
            first_on_line = true;
        }
        if !first_on_line {
            let _ = out.write_all(b" ");
            line_len += 1;
        }
        let _ = out.write_all(word.as_bytes());
        line_len += word_len;
        first_on_line = false;
    }
    if !first_on_line {
        let _ = out.write_all(b"\n");
    }
    para_words.clear();
}

/// Read lines from `reader`, accumulate words into paragraph buffers, and
/// emit word-wrapped output. Blank lines are paragraph separators.
fn fmt_reader(reader: impl BufRead, width: usize, out: &mut impl Write) -> i32 {
    let mut para_words: Vec<String> = Vec::new();
    let mut exit_code = 0;

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    // Blank line = paragraph separator: flush current paragraph, then emit blank line
                    flush_paragraph(&mut para_words, width, out);
                    let _ = out.write_all(b"\n");
                } else {
                    // Accumulate words from this line into the paragraph buffer
                    for word in trimmed.split_whitespace() {
                        para_words.push(word.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("fmt: read error: {e}");
                exit_code = 1;
            }
        }
    }
    // Flush final paragraph (no trailing blank line)
    flush_paragraph(&mut para_words, width, out);
    exit_code
}

fn run(cli: &Cli) -> i32 {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        exit_code |= fmt_reader(reader, cli.width, &mut out);
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    exit_code |= fmt_reader(reader, cli.width, &mut out);
                }
                Err(e) => {
                    eprintln!("fmt: {converted}: {e}");
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
            eprintln!("fmt: {e}");
            return 2;
        }
    };
    run(&cli)
}
