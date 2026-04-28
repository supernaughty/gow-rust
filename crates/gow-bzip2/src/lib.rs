//! `uu_bzip2`: GNU `bzip2`/`bunzip2` — compress and decompress files using the bzip2 algorithm.
//!
//! Mode is determined by argv[0] (`bunzip2`) or by flags (`-d`/`--decompress`).
//! Flags: -d (decompress), -c (stdout), -k (keep original), files... (or stdin if empty).

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use bzip2::read::MultiBzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

/// Operation mode derived from invocation name or flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Compress,
    Decompress,
}

/// GNU bzip2/bunzip2 — Windows port.
#[derive(Parser, Debug)]
#[command(
    name = "bzip2",
    about = "GNU bzip2/bunzip2 — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help, help = "Print help")]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version, help = "Print version")]
    version: Option<bool>,

    /// Decompress (equivalent to bunzip2)
    #[arg(short = 'd', long = "decompress", action = ArgAction::SetTrue)]
    decompress: bool,

    /// Write output to stdout; keep original file
    #[arg(short = 'c', long = "stdout", action = ArgAction::SetTrue)]
    stdout: bool,

    /// Keep (don't delete) original file
    #[arg(short = 'k', long = "keep", action = ArgAction::SetTrue)]
    keep: bool,

    /// Input files (omit or use "-" for stdin)
    #[arg(action = ArgAction::Append, trailing_var_arg = true)]
    files: Vec<String>,
}

/// Detect the operation mode from the argv[0] invocation name and -d flag.
fn detect_mode(invoked_as: &str, decompress_flag: bool) -> Mode {
    if invoked_as == "bunzip2" || decompress_flag {
        Mode::Decompress
    } else {
        Mode::Compress
    }
}

/// Compress bytes from `input` into `output` using bzip2.
fn compress_stream<R: Read, W: Write>(mut input: R, output: W) -> Result<()> {
    let mut encoder = BzEncoder::new(output, Compression::default());
    io::copy(&mut input, &mut encoder).context("bzip2: compression I/O error")?;
    encoder.finish().context("bzip2: failed to finish compression stream")?;
    Ok(())
}

/// Decompress bzip2 bytes from `input` into `output`.
/// Uses MultiBzDecoder for real-world multi-stream compatibility (pbzip2, Wikipedia dumps).
fn decompress_stream<R: Read, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut decoder = MultiBzDecoder::new(input);
    io::copy(&mut decoder, &mut output).context("bzip2: decompression I/O error")?;
    Ok(())
}

/// Derive the output path for compression: appends ".bz2".
fn output_path_compress(input: &str) -> String {
    format!("{input}.bz2")
}

/// Derive the output path for decompression: strips ".bz2" suffix.
/// Returns None if the file doesn't end in ".bz2".
fn output_path_decompress(input: &str) -> Option<String> {
    input.strip_suffix(".bz2").map(|s| s.to_string())
}

/// Core bzip2 runner: processes CLI arguments and performs compress/decompress.
fn run(cli: Cli, invoked_as: &str) -> Result<i32> {
    let mode = detect_mode(invoked_as, cli.decompress);
    let stdout_mode = cli.stdout;
    let keep = cli.keep || stdout_mode; // -c implies keeping the original
    let mut exit_code = 0i32;

    // If no files given, operate on stdin → stdout
    if cli.files.is_empty() {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let result = match mode {
            Mode::Compress => compress_stream(stdin.lock(), &mut stdout),
            Mode::Decompress => decompress_stream(stdin.lock(), &mut stdout),
        };
        if let Err(e) = result {
            eprintln!("bzip2: (stdin): {e}");
            return Ok(1);
        }
        return Ok(0);
    }

    for file in &cli.files {
        // Handle "-" as stdin → stdout
        if file == "-" {
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            let result = match mode {
                Mode::Compress => compress_stream(stdin.lock(), &mut stdout),
                Mode::Decompress => decompress_stream(stdin.lock(), &mut stdout),
            };
            if let Err(e) = result {
                eprintln!("bzip2: -: {e}");
                exit_code = 1;
            }
            continue;
        }

        let converted = gow_core::path::try_convert_msys_path(file);

        match mode {
            Mode::Compress => {
                let out_path = output_path_compress(&converted);

                if stdout_mode {
                    // -c: compress to stdout, keep original
                    match File::open(&converted) {
                        Ok(f) => {
                            let mut stdout = io::stdout();
                            if let Err(e) = compress_stream(f, &mut stdout) {
                                eprintln!("bzip2: {converted}: {e}");
                                exit_code = 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("bzip2: {converted}: {e}");
                            exit_code = 1;
                        }
                    }
                } else {
                    // Normal compress: input → input.bz2, remove input unless -k
                    match File::open(&converted) {
                        Ok(f) => {
                            match File::create(&out_path) {
                                Ok(out_file) => {
                                    if let Err(e) = compress_stream(f, out_file) {
                                        eprintln!("bzip2: {converted}: {e}");
                                        // Remove partial output on error
                                        let _ = fs::remove_file(&out_path);
                                        exit_code = 1;
                                        continue;
                                    }
                                    if !keep {
                                        if let Err(e) = fs::remove_file(&converted) {
                                            eprintln!("bzip2: {converted}: cannot remove: {e}");
                                            exit_code = 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("bzip2: {out_path}: {e}");
                                    exit_code = 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("bzip2: {converted}: {e}");
                            exit_code = 1;
                        }
                    }
                }
            }

            Mode::Decompress => {
                // Derive output filename by stripping ".bz2"
                let out_path_opt = output_path_decompress(&converted);

                if stdout_mode {
                    // -c: decompress to stdout, keep original
                    match File::open(&converted) {
                        Ok(f) => {
                            let mut stdout = io::stdout();
                            if let Err(e) = decompress_stream(f, &mut stdout) {
                                eprintln!("bzip2: {converted}: {e}");
                                exit_code = 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("bzip2: {converted}: {e}");
                            exit_code = 1;
                        }
                    }
                } else {
                    let out_path = match out_path_opt {
                        Some(p) => p,
                        None => {
                            eprintln!(
                                "bzip2: {converted}: unknown suffix -- ignored"
                            );
                            exit_code = 1;
                            continue;
                        }
                    };

                    match File::open(&converted) {
                        Ok(f) => {
                            match File::create(&out_path) {
                                Ok(out_file) => {
                                    if let Err(e) = decompress_stream(f, out_file) {
                                        eprintln!("bzip2: {converted}: {e}");
                                        // Remove partial output on error
                                        let _ = fs::remove_file(&out_path);
                                        exit_code = 1;
                                        continue;
                                    }
                                    if !keep {
                                        if let Err(e) = fs::remove_file(&converted) {
                                            eprintln!("bzip2: {converted}: cannot remove: {e}");
                                            exit_code = 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("bzip2: {out_path}: {e}");
                                    exit_code = 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("bzip2: {converted}: {e}");
                            exit_code = 1;
                        }
                    }
                }
            }
        }
    }

    Ok(exit_code)
}

/// Public entry point — called from main.rs.
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args_vec: Vec<OsString> = args.into_iter().collect();

    // argv[0] mode switching: detect invocation name for bunzip2 dispatch
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
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("bzip2: {e}");
            return 2;
        }
    };

    match run(cli, &invoked_as) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("bzip2: {e}");
            1
        }
    }
}
