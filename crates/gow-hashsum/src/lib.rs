use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use digest::Digest;

#[derive(Parser, Debug)]
#[command(
    name = "md5sum",
    about = "GNU md5sum/sha1sum/sha256sum — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Read sums from FILEs and check them
    #[arg(short = 'c', long = "check", action = ArgAction::SetTrue)]
    check: bool,

    /// Read in binary mode (accepted for GNU compat; output unchanged from text mode)
    #[arg(short = 'b', long = "binary", action = ArgAction::SetTrue)]
    binary: bool,

    /// Read in text mode (default, accepted for GNU compat)
    #[arg(short = 't', long = "text", action = ArgAction::SetTrue)]
    text: bool,

    /// Files to hash (reads stdin when empty)
    files: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Algo {
    Md5,
    Sha1,
    Sha256,
}

fn detect_algo(invoked_as: &str) -> Algo {
    match invoked_as {
        "sha1sum" => Algo::Sha1,
        "sha256sum" => Algo::Sha256,
        // "md5sum" and any other invocation name default to MD5
        _ => Algo::Md5,
    }
}

fn algo_name(a: Algo) -> &'static str {
    match a {
        Algo::Md5 => "md5sum",
        Algo::Sha1 => "sha1sum",
        Algo::Sha256 => "sha256sum",
    }
}

fn hash_reader<D: Digest, R: Read>(mut reader: R) -> io::Result<Vec<u8>> {
    let mut hasher = D::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

fn hash_with_algo<R: Read>(algo: Algo, reader: R) -> io::Result<String> {
    let bytes = match algo {
        Algo::Md5 => hash_reader::<md5::Md5, _>(reader)?,
        Algo::Sha1 => hash_reader::<sha1::Sha1, _>(reader)?,
        Algo::Sha256 => hash_reader::<sha2::Sha256, _>(reader)?,
    };
    Ok(hex::encode(&bytes))
}

fn run_hash_mode(algo: Algo, files: &[String]) -> i32 {
    let mut exit = 0;
    if files.is_empty() {
        let stdin = io::stdin();
        match hash_with_algo(algo, stdin.lock()) {
            Ok(hex_str) => println!("{hex_str}  -"),
            Err(e) => {
                eprintln!("{}: stdin: {e}", algo_name(algo));
                exit = 1;
            }
        }
        return exit;
    }
    for f in files {
        let converted = gow_core::path::try_convert_msys_path(f);
        match File::open(Path::new(&converted)) {
            Ok(file) => match hash_with_algo(algo, file) {
                Ok(hex_str) => println!("{hex_str}  {converted}"),
                Err(e) => {
                    eprintln!("{}: {converted}: {e}", algo_name(algo));
                    exit = 1;
                }
            },
            Err(e) => {
                eprintln!("{}: {converted}: {e}", algo_name(algo));
                exit = 1;
            }
        }
    }
    exit
}

/// Parse a single check-file line.
///
/// Returns `(expected_hex_lowercase, filename)` or `None` if the line is
/// blank, a comment, or malformed.
///
/// Accepted formats:
/// - GNU text mode:   `<hex>  <name>`   (two spaces)
/// - GNU binary mode: `<hex> *<name>`   (one space + asterisk)
fn parse_check_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // GNU text mode: two spaces as separator
    if let Some((hex_part, name_part)) = trimmed.split_once("  ") {
        if !hex_part.is_empty()
            && hex_part.chars().all(|c| c.is_ascii_hexdigit())
            && !name_part.is_empty()
        {
            return Some((hex_part.to_lowercase(), name_part.to_string()));
        }
    }

    // GNU binary mode: "<hex> *<name>"
    if let Some((hex_part, rest)) = trimmed.split_once(' ') {
        if rest.starts_with('*') {
            let name_part = &rest[1..];
            if !hex_part.is_empty()
                && hex_part.chars().all(|c| c.is_ascii_hexdigit())
                && !name_part.is_empty()
            {
                return Some((hex_part.to_lowercase(), name_part.to_string()));
            }
        }
    }

    None
}

fn process_check_lines(
    reader: Box<dyn BufRead>,
    source_label: &str,
    algo: Algo,
    exit: &mut i32,
) {
    for (lineno, line_res) in reader.lines().enumerate() {
        let line = match line_res {
            Ok(l) => l,
            Err(e) => {
                eprintln!(
                    "{}: {source_label}: line {}: {e}",
                    algo_name(algo),
                    lineno + 1
                );
                *exit = 1;
                return;
            }
        };

        let Some((expected_hex, name)) = parse_check_line(&line) else {
            if !line.trim().is_empty() && !line.starts_with('#') {
                eprintln!(
                    "{}: {source_label}: improperly formatted checksum line",
                    algo_name(algo)
                );
                *exit = 1;
            }
            continue;
        };

        // Validate hex length matches this algorithm's output width
        let expected_len = match algo {
            Algo::Md5 => 32,
            Algo::Sha1 => 40,
            Algo::Sha256 => 64,
        };
        if expected_hex.len() != expected_len {
            eprintln!(
                "{}: {source_label}: hex length {} does not match {} (expected {})",
                algo_name(algo),
                expected_hex.len(),
                algo_name(algo),
                expected_len
            );
            *exit = 1;
            continue;
        }

        // Recompute hash for the listed file and compare
        let converted = gow_core::path::try_convert_msys_path(&name);
        match File::open(Path::new(&converted)) {
            Ok(file) => match hash_with_algo(algo, file) {
                Ok(actual) if actual == expected_hex => {
                    println!("{name}: OK");
                }
                Ok(_) => {
                    println!("{name}: FAILED");
                    *exit = 1;
                }
                Err(_) => {
                    println!("{name}: FAILED open or read");
                    *exit = 1;
                }
            },
            Err(_) => {
                println!("{name}: FAILED open or read");
                *exit = 1;
            }
        }
    }
}

fn run_check_mode(algo: Algo, files: &[String]) -> i32 {
    let mut exit = 0;

    if files.is_empty() {
        let stdin = io::stdin();
        let reader: Box<dyn BufRead> = Box::new(stdin.lock());
        process_check_lines(reader, "stdin", algo, &mut exit);
    } else {
        for f in files {
            let converted = gow_core::path::try_convert_msys_path(f);
            match File::open(Path::new(&converted)) {
                Ok(file) => {
                    let reader: Box<dyn BufRead> = Box::new(BufReader::new(file));
                    process_check_lines(reader, &converted, algo, &mut exit);
                }
                Err(e) => {
                    eprintln!("{}: {converted}: {e}", algo_name(algo));
                    exit = 1;
                }
            }
        }
    }
    exit
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args_vec: Vec<OsString> = args.into_iter().collect();

    // Detect which hash algorithm to use based on argv[0]
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

    let algo = detect_algo(&invoked_as);

    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {e}", algo_name(algo));
            return 2;
        }
    };

    if cli.check {
        run_check_mode(algo, &cli.files)
    } else {
        run_hash_mode(algo, &cli.files)
    }
}
