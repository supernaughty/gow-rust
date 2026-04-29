use std::ffi::OsString;
use std::io::{self, Read, Write};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "split",
    about = "GNU split — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Split every N lines (default 1000)
    #[arg(short = 'l', long = "lines", value_name = "NUMBER")]
    lines: Option<usize>,

    /// Split every SIZE bytes (supports K, M, G suffix: 1K=1024, 1M=1048576)
    #[arg(short = 'b', long = "bytes", value_name = "SIZE")]
    bytes: Option<String>,

    /// Split into N equal chunks by byte count
    #[arg(short = 'n', long = "number", value_name = "CHUNKS")]
    chunks: Option<usize>,

    /// Use N-character alphabetic suffixes (default 2)
    #[arg(short = 'a', long = "suffix-length", default_value = "2")]
    suffix_len: usize,

    /// Input file (omit or use - for stdin)
    input: Option<String>,

    /// Output file prefix (default: x)
    prefix: Option<String>,
}

/// Parse a byte-size string with optional K/M/G suffix.
fn parse_bytes(s: &str) -> Option<usize> {
    let s = s.trim();
    let (num_str, mult) = if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len() - 1], 1024 * 1024 * 1024usize)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1024 * 1024)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1024)
    } else {
        (s, 1)
    };
    num_str.parse::<usize>().ok().map(|n| n * mult)
}

/// Advance to the next alphabetic suffix in place.
/// Suffixes cycle from "aa" → "ab" → ... → "az" → "ba" → ... → "zz" → "aaa" → ...
fn next_suffix(suffix: &mut Vec<u8>) {
    let mut i = suffix.len() - 1;
    loop {
        if suffix[i] < b'z' {
            suffix[i] += 1;
            return;
        }
        suffix[i] = b'a';
        if i == 0 {
            // All z's — extend: zz → aaa, zzz → aaaa, etc.
            suffix.iter_mut().for_each(|c| *c = b'a');
            suffix.push(b'a');
            return;
        }
        i -= 1;
    }
}

/// Write a chunk of bytes to a new output file named prefix+suffix.
/// Advances suffix after writing. Returns 1 on error, 0 on success.
fn write_chunk(data: &[u8], prefix: &str, suffix: &mut Vec<u8>) -> i32 {
    let suffix_str = String::from_utf8_lossy(suffix).to_string();
    let filename = format!("{}{}", prefix, suffix_str);
    match std::fs::File::create(&filename) {
        Ok(mut f) => match f.write_all(data) {
            Ok(_) => {
                next_suffix(suffix);
                0
            }
            Err(e) => {
                eprintln!("split: {filename}: {e}");
                1
            }
        },
        Err(e) => {
            eprintln!("split: {filename}: {e}");
            1
        }
    }
}

fn run(cli: &Cli) -> i32 {
    // Validate suffix_len
    if cli.suffix_len == 0 {
        eprintln!("split: suffix length must be > 0");
        return 1;
    }

    // Parse byte size if -b given
    let byte_size: Option<usize> = if let Some(ref bs) = cli.bytes {
        match parse_bytes(bs) {
            Some(n) if n > 0 => Some(n),
            Some(_) => {
                eprintln!("split: invalid number of bytes: '{}'", bs);
                return 1;
            }
            None => {
                eprintln!("split: invalid number of bytes: '{}'", bs);
                return 1;
            }
        }
    } else {
        None
    };

    // Read input into memory
    let input: Vec<u8> = if cli.input.is_none() || cli.input.as_deref() == Some("-") {
        let mut buf = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut buf) {
            eprintln!("split: stdin: {e}");
            return 1;
        }
        buf
    } else {
        let path = gow_core::path::try_convert_msys_path(cli.input.as_ref().unwrap());
        match std::fs::read(&path) {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("split: {path}: {e}");
                return 1;
            }
        }
    };

    let prefix = cli.prefix.clone().unwrap_or_else(|| "x".to_string());
    let mut suffix: Vec<u8> = vec![b'a'; cli.suffix_len];

    if let Some(byte_count) = byte_size {
        // -b mode: split by byte count
        for chunk in input.chunks(byte_count) {
            if write_chunk(chunk, &prefix, &mut suffix) != 0 {
                return 1;
            }
        }
    } else if let Some(chunks) = cli.chunks {
        // -n mode: split into N equal chunks by byte count
        if chunks == 0 {
            eprintln!("split: invalid number of chunks: '0'");
            return 1;
        }
        let total = input.len();
        // If the input is empty, create one empty chunk per requested chunk, or just one.
        if total == 0 {
            if write_chunk(&[], &prefix, &mut suffix) != 0 {
                return 1;
            }
        } else {
            let chunk_size = (total + chunks - 1) / chunks; // ceiling division
            for chunk in input.chunks(chunk_size) {
                if write_chunk(chunk, &prefix, &mut suffix) != 0 {
                    return 1;
                }
            }
        }
    } else {
        // -l mode (or default 1000 lines)
        let line_count = cli.lines.unwrap_or(1000);
        if line_count == 0 {
            eprintln!("split: invalid number of lines: '0'");
            return 1;
        }

        // Stream line-by-line to avoid reading everything twice, but input is already in memory.
        // Split input into lines (preserving newlines).
        let mut current_chunk: Vec<u8> = Vec::new();
        let mut line_num: usize = 0;

        // Iterate over lines in the input buffer
        let mut iter = input.as_slice();
        while !iter.is_empty() {
            // Find the next newline
            let (line_bytes, rest) = if let Some(pos) = iter.iter().position(|&b| b == b'\n') {
                (&iter[..pos + 1], &iter[pos + 1..])
            } else {
                // Last line without trailing newline
                (iter, &b""[..])
            };

            current_chunk.extend_from_slice(line_bytes);
            line_num += 1;
            iter = rest;

            if line_num == line_count {
                if write_chunk(&current_chunk, &prefix, &mut suffix) != 0 {
                    return 1;
                }
                current_chunk.clear();
                line_num = 0;
            }
        }

        // Write any remaining lines
        if !current_chunk.is_empty() {
            if write_chunk(&current_chunk, &prefix, &mut suffix) != 0 {
                return 1;
            }
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
            eprintln!("split: {e}");
            return 2;
        }
    };
    run(&cli)
}
