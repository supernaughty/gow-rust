use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "fold",
    about = "GNU fold — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Wrap width in bytes (default 80)
    #[arg(short = 'w', long = "width", default_value = "80")]
    width: usize,

    /// Break at the last space within the width window when present
    #[arg(short = 's', long = "spaces", action = ArgAction::SetTrue)]
    spaces: bool,

    /// Count bytes rather than characters (accepted for compatibility; Phase 10 always counts bytes)
    #[arg(short = 'b', long = "bytes", action = ArgAction::SetTrue)]
    bytes: bool,

    /// Files to fold (reads stdin if none provided)
    files: Vec<String>,
}

/// Wrap a single line (without trailing newline) at `width` bytes.
/// With `spaces=true`, tries to break at the last space within the window.
/// Writes wrapped output to `out`, each segment followed by `\n`.
fn wrap_line(line: &[u8], width: usize, spaces: bool, out: &mut impl Write) {
    if line.len() <= width {
        let _ = out.write_all(line);
        let _ = out.write_all(b"\n");
        return;
    }

    let mut pos = 0;
    while pos < line.len() {
        let remaining = line.len() - pos;
        if remaining <= width {
            // Final segment — fits entirely
            let _ = out.write_all(&line[pos..]);
            let _ = out.write_all(b"\n");
            break;
        }

        // We have more than `width` bytes remaining
        let chunk_end = pos + width;

        if spaces {
            // Search for the last space in the window [pos..chunk_end+1].
            // Including chunk_end handles the case where the break point is exactly
            // at the width boundary (e.g., "hello world goodbye" with w=11: space is
            // at index 11 which is chunk_end; we break before it, keeping "hello world").
            let search_end = (chunk_end + 1).min(line.len());
            let chunk = &line[pos..search_end];
            if let Some(space_rel) = chunk.iter().rposition(|&b| b == b' ') {
                let break_at = pos + space_rel;
                if break_at > pos {
                    // Break before the space
                    let _ = out.write_all(&line[pos..break_at]);
                    let _ = out.write_all(b"\n");
                    pos = break_at;
                    continue;
                }
            }
            // No usable space found within window — fall through to hard break
        }

        // Hard break at width
        let _ = out.write_all(&line[pos..chunk_end]);
        let _ = out.write_all(b"\n");
        pos = chunk_end;
    }
}

/// Wrap all lines from a buffered reader and write to stdout.
fn wrap_reader(reader: impl BufRead, width: usize, spaces: bool, out: &mut impl Write) -> i32 {
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                wrap_line(line.as_bytes(), width, spaces, out);
            }
            Err(e) => {
                eprintln!("fold: read error: {e}");
                return 1;
            }
        }
    }
    0
}

fn run(cli: &Cli) -> i32 {
    if cli.width == 0 {
        eprintln!("fold: invalid number of columns: '0'");
        return 1;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        exit_code |= wrap_reader(reader, cli.width, cli.spaces, &mut out);
    } else {
        for file_path in &cli.files {
            let converted = gow_core::path::try_convert_msys_path(file_path);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    exit_code |= wrap_reader(reader, cli.width, cli.spaces, &mut out);
                }
                Err(e) => {
                    eprintln!("fold: {converted}: {e}");
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
            eprintln!("fold: {e}");
            return 2;
        }
    };
    run(&cli)
}
