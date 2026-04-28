use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Parser, Debug)]
#[command(
    name = "gzip",
    about = "GNU gzip/gunzip/zcat — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Decompress (force decompress mode)
    #[arg(short = 'd', long = "decompress", action = ArgAction::SetTrue)]
    decompress: bool,

    /// Write output to stdout, keep original files
    #[arg(short = 'c', long = "stdout", action = ArgAction::SetTrue)]
    stdout: bool,

    /// Keep original files (do not delete after compress/decompress)
    #[arg(short = 'k', long = "keep", action = ArgAction::SetTrue)]
    keep: bool,

    /// Input files (zero or more; reads stdin when empty)
    files: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Compress,
    Decompress,
}

fn detect_mode(invoked_as: &str, decompress_flag: bool) -> Mode {
    if invoked_as == "gunzip" || invoked_as == "zcat" || decompress_flag {
        Mode::Decompress
    } else {
        Mode::Compress
    }
}

fn is_stdout_mode(invoked_as: &str, stdout_flag: bool) -> bool {
    invoked_as == "zcat" || stdout_flag
}

fn compress_stream<R: Read, W: Write>(mut input: R, output: W) -> Result<()> {
    let mut encoder = GzEncoder::new(output, Compression::default());
    io::copy(&mut input, &mut encoder).context("compress: io::copy failed")?;
    encoder.finish().context("compress: failed to finalize gzip stream")?;
    Ok(())
}

fn decompress_stream<R: Read, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut decoder = MultiGzDecoder::new(input);
    io::copy(&mut decoder, &mut output).context("decompress: io::copy failed")?;
    Ok(())
}

fn run(cli: &Cli, invoked_as: &str) -> i32 {
    let mode = detect_mode(invoked_as, cli.decompress);
    let stdout_mode = is_stdout_mode(invoked_as, cli.stdout);
    let mut exit_code = 0;

    if cli.files.is_empty() {
        let stdin = io::stdin();
        let stdout = io::stdout();
        match mode {
            Mode::Compress => {
                if let Err(e) = compress_stream(stdin.lock(), stdout.lock()) {
                    eprintln!("gzip: stdin: {e}");
                    return 1;
                }
                return 0;
            }
            Mode::Decompress => {
                if let Err(e) = decompress_stream(stdin.lock(), stdout.lock()) {
                    eprintln!("gzip: stdin: {e}");
                    return 1;
                }
                return 0;
            }
        }
    }

    for file_path in &cli.files {
        let converted = gow_core::path::try_convert_msys_path(file_path);
        let path = Path::new(&converted);

        if stdout_mode {
            // Write all output to stdout; never remove input file
            match File::open(path) {
                Ok(f) => {
                    let stdout = io::stdout();
                    let result = match mode {
                        Mode::Compress => compress_stream(f, stdout.lock()),
                        Mode::Decompress => decompress_stream(f, stdout.lock()),
                    };
                    if let Err(e) = result {
                        eprintln!("gzip: {converted}: {e}");
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("gzip: {converted}: {e}");
                    exit_code = 1;
                }
            }
        } else {
            match mode {
                Mode::Compress => {
                    // Output: <input>.gz
                    let out_path = format!("{converted}.gz");
                    match (File::open(path), File::create(&out_path)) {
                        (Ok(input), Ok(output)) => {
                            if let Err(e) = compress_stream(input, output) {
                                eprintln!("gzip: {converted}: {e}");
                                exit_code = 1;
                                // Remove partial output file
                                let _ = std::fs::remove_file(&out_path);
                            } else if !cli.keep {
                                if let Err(e) = std::fs::remove_file(path) {
                                    eprintln!("gzip: {converted}: {e}");
                                    exit_code = 1;
                                }
                            }
                        }
                        (Err(e), _) => {
                            eprintln!("gzip: {converted}: {e}");
                            exit_code = 1;
                        }
                        (_, Err(e)) => {
                            eprintln!("gzip: {out_path}: {e}");
                            exit_code = 1;
                        }
                    }
                }
                Mode::Decompress => {
                    // Output: strip ".gz" suffix
                    let out_path = if converted.ends_with(".gz") {
                        converted[..converted.len() - 3].to_string()
                    } else {
                        eprintln!("gzip: {converted}: unknown suffix -- ignored");
                        exit_code = 1;
                        continue;
                    };
                    match (File::open(path), File::create(&out_path)) {
                        (Ok(input), Ok(output)) => {
                            if let Err(e) = decompress_stream(input, output) {
                                eprintln!("gzip: {converted}: {e}");
                                exit_code = 1;
                                // Remove partial output file
                                let _ = std::fs::remove_file(&out_path);
                            } else if !cli.keep {
                                if let Err(e) = std::fs::remove_file(path) {
                                    eprintln!("gzip: {converted}: {e}");
                                    exit_code = 1;
                                }
                            }
                        }
                        (Err(e), _) => {
                            eprintln!("gzip: {converted}: {e}");
                            exit_code = 1;
                        }
                        (_, Err(e)) => {
                            eprintln!("gzip: {out_path}: {e}");
                            exit_code = 1;
                        }
                    }
                }
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
        .unwrap_or_default();

    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("gzip: {e}");
            return 2;
        }
    };

    run(&cli, &invoked_as)
}
