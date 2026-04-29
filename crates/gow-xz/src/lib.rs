//! `uu_xz`: GNU `xz`/`unxz` — xz/lzma compression and decompression (R019).
//!
//! Mode detection:
//!   - Default mode: compress (invoked as "xz")
//!   - Decompress mode: invoked as "unxz", or -d/--decompress flag passed
//!   - Stdout mode: -c/--stdout flag — compress/decompress to stdout, keep original
//!   - Keep mode: -k/--keep flag — do not remove the original file after operation
//!
//! Codec: liblzma 0.4.6 (MSVC-safe fork of xz2) with static feature.
//! Compression level: 6 (matches xz CLI default).

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use liblzma::read::XzDecoder;
use liblzma::write::XzEncoder;

// ── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "xz",
    about = "GNU xz/unxz — compress or decompress .xz files.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// Print help
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Decompress (equivalent to unxz)
    #[arg(short = 'd', long = "decompress")]
    decompress: bool,

    /// Write output to stdout; keep original file
    #[arg(short = 'c', long = "stdout")]
    stdout: bool,

    /// Keep original file (do not remove after operation)
    #[arg(short = 'k', long = "keep")]
    keep: bool,

    /// Input files (reads from stdin if empty or "-")
    #[arg(trailing_var_arg = true)]
    files: Vec<String>,
}

// ── Mode ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Compress,
    Decompress,
}

fn detect_mode(invoked_as: &str, decompress_flag: bool) -> Mode {
    if invoked_as == "unxz" || decompress_flag {
        Mode::Decompress
    } else {
        Mode::Compress
    }
}

// ── Core stream transforms ────────────────────────────────────────────────────

/// Compress bytes from `input` into `output` using XzEncoder at level 6.
fn compress_stream<R: Read, W: Write>(mut input: R, output: W) -> Result<()> {
    let mut encoder = XzEncoder::new(output, 6);
    io::copy(&mut input, &mut encoder).context("compress: io::copy failed")?;
    encoder.finish().context("compress: failed to finalize xz stream")?;
    Ok(())
}

/// Decompress bytes from `input` into `output` using XzDecoder.
fn decompress_stream<R: Read, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut decoder = XzDecoder::new_multi_decoder(input);
    io::copy(&mut decoder, &mut output).context("decompress: io::copy failed")?;
    Ok(())
}

// ── Derive output file path ───────────────────────────────────────────────────

/// For compress: append ".xz" suffix.
fn compress_output_path(input_path: &str) -> String {
    format!("{input_path}.xz")
}

/// For decompress: strip ".xz" suffix. Returns error if input doesn't end with ".xz".
fn decompress_output_path(input_path: &str) -> Result<String> {
    if let Some(stem) = input_path.strip_suffix(".xz") {
        Ok(stem.to_string())
    } else {
        Err(anyhow::anyhow!(
            "unknown suffix -- ignored (expected .xz extension)"
        ))
    }
}

// ── Main run logic ────────────────────────────────────────────────────────────

fn run(cli: Cli, invoked_as: &str) -> i32 {
    let mode = detect_mode(invoked_as, cli.decompress);
    let mut exit_code = 0i32;

    // stdin/stdout path: no file arguments or explicit stdout mode with empty files
    if cli.files.is_empty() {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let result = match mode {
            Mode::Compress => compress_stream(stdin.lock(), stdout.lock()),
            Mode::Decompress => decompress_stream(stdin.lock(), stdout.lock()),
        };
        if let Err(e) = result {
            eprintln!("xz: stdin: {e}");
            exit_code = 1;
        }
        return exit_code;
    }

    for file_arg in &cli.files {
        // Handle "-" as stdin → stdout
        if file_arg == "-" {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let result = match mode {
                Mode::Compress => compress_stream(stdin.lock(), stdout.lock()),
                Mode::Decompress => decompress_stream(stdin.lock(), stdout.lock()),
            };
            if let Err(e) = result {
                eprintln!("xz: -: {e}");
                exit_code = 1;
            }
            continue;
        }

        let converted = gow_core::path::try_convert_msys_path(file_arg);

        // Stdout mode: compress/decompress to stdout, never remove original
        if cli.stdout {
            let input_file = match File::open(&converted) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("xz: {converted}: {e}");
                    exit_code = 1;
                    continue;
                }
            };
            let stdout = io::stdout();
            let result = match mode {
                Mode::Compress => compress_stream(input_file, stdout.lock()),
                Mode::Decompress => decompress_stream(input_file, stdout.lock()),
            };
            if let Err(e) = result {
                eprintln!("xz: {converted}: {e}");
                exit_code = 1;
            }
            continue;
        }

        // File-to-file mode: derive output path
        let output_path = match mode {
            Mode::Compress => compress_output_path(&converted),
            Mode::Decompress => match decompress_output_path(&converted) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("xz: {converted}: {e}");
                    exit_code = 1;
                    continue;
                }
            },
        };

        // Open input
        let input_file = match File::open(&converted) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("xz: {converted}: {e}");
                exit_code = 1;
                continue;
            }
        };

        // Create output file
        let output_file = match File::create(&output_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("xz: {output_path}: {e}");
                exit_code = 1;
                continue;
            }
        };

        let result = match mode {
            Mode::Compress => compress_stream(input_file, output_file),
            Mode::Decompress => decompress_stream(input_file, output_file),
        };

        if let Err(e) = result {
            eprintln!("xz: {converted}: {e}");
            // Remove incomplete output file on error
            let _ = fs::remove_file(&output_path);
            exit_code = 1;
            continue;
        }

        // Remove original unless -k/--keep
        if !cli.keep {
            if let Err(e) = fs::remove_file(&converted) {
                eprintln!("xz: {converted}: {e}");
                exit_code = 1;
            }
        }
    }

    exit_code
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args_vec: Vec<OsString> = args.into_iter().collect();

    // Detect argv[0] for mode switching (xz vs unxz)
    let invoked_as = args_vec
        .first()
        .map(|s| {
            Path::new(s)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        })
        .unwrap_or_default();

    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    run(cli, &invoked_as)
}
