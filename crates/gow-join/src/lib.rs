use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "join",
    about = "GNU join — Windows port. Input files must be sorted on the join field.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Join field in FILE1 (1-based, default 1)
    #[arg(short = '1', value_name = "FIELD", default_value = "1")]
    field1: usize,

    /// Join field in FILE2 (1-based, default 1)
    #[arg(short = '2', value_name = "FIELD", default_value = "1")]
    field2: usize,

    /// Field separator character (default: whitespace)
    #[arg(short = 't', value_name = "CHAR")]
    separator: Option<String>,

    /// Print unpairable lines from FILE (1 or 2)
    #[arg(short = 'a', value_name = "FILENUM")]
    print_unpaired: Option<u8>,

    /// Print only unpairable lines from FILE (1 or 2) — suppress joined output
    #[arg(short = 'v', value_name = "FILENUM")]
    only_unpaired: Option<u8>,

    /// FILE1 and FILE2 (use - for stdin)
    files: Vec<String>,
}

/// Extract a field (1-based) from a line using the given separator.
/// Returns an empty string if the field index is out of range.
fn get_field<'a>(line: &'a str, field: usize, sep: Option<char>) -> &'a str {
    if field == 0 {
        return "";
    }
    if let Some(c) = sep {
        line.splitn(field + 1, c).nth(field - 1).unwrap_or("")
    } else {
        line.split_whitespace().nth(field - 1).unwrap_or("")
    }
}

/// Return all fields except the join field, joined by the separator.
fn other_fields(line: &str, join_field: usize, sep: Option<char>) -> String {
    if let Some(c) = sep {
        let parts: Vec<&str> = line.split(c).collect();
        parts
            .iter()
            .enumerate()
            .filter(|(i, _)| i + 1 != join_field)
            .map(|(_, s)| *s)
            .collect::<Vec<_>>()
            .join(&c.to_string())
    } else {
        line.split_whitespace()
            .enumerate()
            .filter(|(i, _)| i + 1 != join_field)
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Open a file path as a BufReader. Treats "-" as stdin.
fn open_input(path: &str) -> Result<Box<dyn BufRead>, String> {
    if path == "-" {
        Ok(Box::new(BufReader::new(io::stdin())))
    } else {
        let converted = gow_core::path::try_convert_msys_path(path);
        match File::open(&converted) {
            Ok(f) => Ok(Box::new(BufReader::new(f))),
            Err(e) => Err(format!("join: {converted}: {e}")),
        }
    }
}

/// Read the next non-empty line from a BufReader (strips trailing `\r\n` or `\n`).
fn read_line(reader: &mut Box<dyn BufRead>) -> Option<String> {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return None, // EOF
            Ok(_) => {
                // Strip trailing newline(s)
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                return Some(line);
            }
            Err(_) => return None,
        }
    }
}

/// Print a joined output line: key + file1 other fields + file2 other fields.
fn print_joined(
    out: &mut impl Write,
    key: &str,
    f1_other: &str,
    f2_other: &str,
    sep: Option<char>,
) {
    let sep_str = sep.map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());
    let mut parts: Vec<&str> = vec![key];
    if !f1_other.is_empty() {
        parts.push(f1_other);
    }
    if !f2_other.is_empty() {
        parts.push(f2_other);
    }
    let _ = writeln!(out, "{}", parts.join(&sep_str));
}

fn run(cli: &Cli) -> i32 {
    // Validate: need exactly 2 files
    if cli.files.len() != 2 {
        eprintln!("join: invalid usage: need exactly 2 files");
        return 2;
    }

    let path1 = &cli.files[0];
    let path2 = &cli.files[1];

    let mut reader1 = match open_input(path1) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let mut reader2 = match open_input(path2) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };

    let sep: Option<char> = cli
        .separator
        .as_ref()
        .and_then(|s| s.chars().next());

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Initial reads
    let mut line1: Option<String> = read_line(&mut reader1);
    let mut line2: Option<String> = read_line(&mut reader2);

    // Sort-order tracking
    let mut prev_key1: Option<String> = None;
    let mut prev_key2: Option<String> = None;

    loop {
        match (&line1, &line2) {
            (None, None) => break,
            (Some(l1), None) => {
                // File2 exhausted; drain file1
                let key1 = get_field(l1, cli.field1, sep).to_string();
                let should_print = cli.print_unpaired == Some(1) || cli.only_unpaired == Some(1);
                if should_print {
                    let f1_other = other_fields(l1, cli.field1, sep);
                    print_joined(&mut out, &key1, &f1_other, "", sep);
                }
                line1 = read_line(&mut reader1);
                // Continue draining
                while let Some(ref l1) = line1.clone() {
                    let key1 = get_field(l1, cli.field1, sep).to_string();
                    if should_print {
                        let f1_other = other_fields(l1, cli.field1, sep);
                        print_joined(&mut out, &key1, &f1_other, "", sep);
                    }
                    line1 = read_line(&mut reader1);
                }
                break;
            }
            (None, Some(l2)) => {
                // File1 exhausted; drain file2
                let key2 = get_field(l2, cli.field2, sep).to_string();
                let should_print = cli.print_unpaired == Some(2) || cli.only_unpaired == Some(2);
                if should_print {
                    let f2_other = other_fields(l2, cli.field2, sep);
                    print_joined(&mut out, &key2, "", &f2_other, sep);
                }
                line2 = read_line(&mut reader2);
                while let Some(ref l2) = line2.clone() {
                    let key2 = get_field(l2, cli.field2, sep).to_string();
                    if should_print {
                        let f2_other = other_fields(l2, cli.field2, sep);
                        print_joined(&mut out, &key2, "", &f2_other, sep);
                    }
                    line2 = read_line(&mut reader2);
                }
                break;
            }
            (Some(l1), Some(l2)) => {
                let key1 = get_field(l1, cli.field1, sep).to_string();
                let key2 = get_field(l2, cli.field2, sep).to_string();

                // Sort-order warnings (T-11-03-03 mitigation)
                if let Some(ref pk1) = prev_key1 {
                    if key1.as_str() < pk1.as_str() {
                        eprintln!("join: file 1 is not in sorted order");
                    }
                }
                if let Some(ref pk2) = prev_key2 {
                    if key2.as_str() < pk2.as_str() {
                        eprintln!("join: file 2 is not in sorted order");
                    }
                }

                match key1.as_str().cmp(key2.as_str()) {
                    std::cmp::Ordering::Equal => {
                        // Keys match — output joined line (unless only_unpaired is set)
                        if cli.only_unpaired.is_none() {
                            let f1_other = other_fields(l1, cli.field1, sep);
                            let f2_other = other_fields(l2, cli.field2, sep);
                            print_joined(&mut out, &key1, &f1_other, &f2_other, sep);
                        }
                        prev_key1 = Some(key1);
                        prev_key2 = Some(key2);
                        line1 = read_line(&mut reader1);
                        line2 = read_line(&mut reader2);
                    }
                    std::cmp::Ordering::Less => {
                        // key1 < key2: line1 is unmatched
                        if cli.print_unpaired == Some(1) || cli.only_unpaired == Some(1) {
                            let f1_other = other_fields(l1, cli.field1, sep);
                            print_joined(&mut out, &key1, &f1_other, "", sep);
                        }
                        prev_key1 = Some(key1);
                        line1 = read_line(&mut reader1);
                    }
                    std::cmp::Ordering::Greater => {
                        // key1 > key2: line2 is unmatched
                        if cli.print_unpaired == Some(2) || cli.only_unpaired == Some(2) {
                            let f2_other = other_fields(l2, cli.field2, sep);
                            print_joined(&mut out, &key2, "", &f2_other, sep);
                        }
                        prev_key2 = Some(key2);
                        line2 = read_line(&mut reader2);
                    }
                }
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
            eprintln!("join: {e}");
            return 2;
        }
    };
    run(&cli)
}
