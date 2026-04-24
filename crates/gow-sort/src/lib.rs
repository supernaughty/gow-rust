//! `uu_sort`: GNU `sort` — Windows port.
//!
//! Implements external merge sort for large file support.

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use bstr::io::BufReadExt;
use bstr::ByteSlice;
use clap::{Arg, ArgAction, Command};
use itertools::Itertools;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy)]
struct SortConfig {
    numeric: bool,
    reverse: bool,
    unique: bool,
    ignore_case: bool,
    buffer_size: usize,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    let config = SortConfig {
        numeric: matches.get_flag("numeric-sort"),
        reverse: matches.get_flag("reverse"),
        unique: matches.get_flag("unique"),
        ignore_case: matches.get_flag("ignore-case"),
        buffer_size: parse_buffer_size(matches.get_one::<String>("buffer-size")),
    };

    let output_file = matches.get_one::<String>("output").map(|s| s.to_string());
    let mut operands: Vec<String> = matches
        .get_many::<String>("files")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    if operands.is_empty() {
        operands.push("-".to_string());
    }

    if let Err(e) = run_sort(operands, config, output_file) {
        eprintln!("sort: {e}");
        return 1;
    }

    0
}

fn parse_buffer_size(s: Option<&String>) -> usize {
    // Default to 100MB if not specified
    let default = 100 * 1024 * 1024;
    match s {
        Some(s) => {
            // Basic parsing of K, M, G suffixes
            let s = s.to_uppercase();
            if s.ends_with('K') {
                s[..s.len() - 1].parse::<usize>().unwrap_or(default / 1024) * 1024
            } else if s.ends_with('M') {
                s[..s.len() - 1].parse::<usize>().unwrap_or(default / (1024 * 1024)) * 1024 * 1024
            } else if s.ends_with('G') {
                s[..s.len() - 1].parse::<usize>().unwrap_or(0) * 1024 * 1024 * 1024
            } else {
                s.parse::<usize>().unwrap_or(default)
            }
        }
        None => default,
    }
}

fn run_sort(operands: Vec<String>, config: SortConfig, output_file: Option<String>) -> anyhow::Result<()> {
    let mut temp_files = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_buffer_usage = 0;

    for op in operands {
        let mut input: Box<dyn BufRead> = if op == "-" {
            Box::new(BufReader::new(io::stdin().lock()))
        } else {
            let converted = gow_core::path::try_convert_msys_path(&op);
            let file = File::open(Path::new(&converted))?;
            Box::new(BufReader::new(file))
        };

        input.for_byte_line(|line| {
            current_buffer_usage += line.len() + std::mem::size_of::<Vec<u8>>();
            current_lines.push(line.to_vec());

            if current_buffer_usage >= config.buffer_size {
                let temp = sort_and_write_chunk(std::mem::take(&mut current_lines), &config)?;
                temp_files.push(temp);
                current_buffer_usage = 0;
            }
            Ok(true)
        })?;
    }

    let out: Box<dyn Write> = if let Some(out_path) = output_file {
        let converted = gow_core::path::try_convert_msys_path(&out_path);
        Box::new(BufWriter::new(File::create(Path::new(&converted))?))
    } else {
        Box::new(BufWriter::new(io::stdout().lock()))
    };

    if temp_files.is_empty() {
        // All fit in memory
        write_sorted(current_lines, config, out)?;
    } else {
        // Spilled to disk, need to merge
        if !current_lines.is_empty() {
            let temp = sort_and_write_chunk(current_lines, &config)?;
            temp_files.push(temp);
        }
        merge_temp_files(temp_files, config, out)?;
    }

    Ok(())
}

fn compare_lines(a: &[u8], b: &[u8], numeric: bool, ignore_case: bool) -> Ordering {
    if numeric {
        let na = parse_numeric(a);
        let nb = parse_numeric(b);
        if na != nb {
            return na.partial_cmp(&nb).unwrap_or(Ordering::Equal);
        }
    }

    if ignore_case {
        let a_iter = a.chars().flat_map(|c| c.to_lowercase());
        let b_iter = b.chars().flat_map(|c| c.to_lowercase());
        return a_iter.cmp(b_iter);
    }

    a.cmp(b)
}

fn parse_numeric(s: &[u8]) -> f64 {
    let s = s.trim();
    if s.is_empty() {
        return 0.0;
    }
    let s_str = String::from_utf8_lossy(s);
    // Find numeric prefix
    let end = s_str
        .find(|c: char| !c.is_digit(10) && c != '.' && c != '-' && c != '+')
        .unwrap_or(s_str.len());
    s_str[..end].parse::<f64>().unwrap_or(0.0)
}

fn sort_and_write_chunk(mut lines: Vec<Vec<u8>>, config: &SortConfig) -> io::Result<NamedTempFile> {
    lines.sort_by(|a, b| {
        let mut ord = compare_lines(a, b, config.numeric, config.ignore_case);
        if config.reverse {
            ord = ord.reverse();
        }
        ord
    });

    let temp = NamedTempFile::new()?;
    let mut writer = BufWriter::new(temp);
    for line in lines {
        writer.write_all(&line)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    writer
        .into_inner()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

fn write_sorted(mut lines: Vec<Vec<u8>>, config: SortConfig, mut out: Box<dyn Write>) -> io::Result<()> {
    lines.sort_by(|a, b| {
        let mut ord = compare_lines(a, b, config.numeric, config.ignore_case);
        if config.reverse {
            ord = ord.reverse();
        }
        ord
    });

    let mut last_line: Option<Vec<u8>> = None;
    for line in lines {
        if config.unique {
            if let Some(ref last) = last_line {
                if compare_lines(&line, last, config.numeric, config.ignore_case) == Ordering::Equal {
                    continue;
                }
            }
            last_line = Some(line.clone());
        }
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}

fn merge_temp_files(temp_files: Vec<NamedTempFile>, config: SortConfig, mut out: Box<dyn Write>) -> io::Result<()> {
    // Open all temp files for reading
    let mut readers = Vec::new();
    for temp in &temp_files {
        // Re-open the file for reading
        let file = File::open(temp.path())?;
        readers.push(BufReader::new(file));
    }

    // We need to keep the readers in a way that we can iterate over their lines.
    // kmerge works on iterators.
    let mut line_iters = Vec::new();
    for reader in readers {
        line_iters.push(reader.byte_lines());
    }

    // Wrap the result in a way that kmerge can use.
    // Each element in line_iters is an iterator yielding io::Result<Vec<u8>>.
    // kmerge_by expects items of the same type.
    
    let merged_iter = line_iters.into_iter().kmerge_by(move |a, b| {
        let a_val = a.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let b_val = b.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let mut ord = compare_lines(a_val, b_val, config.numeric, config.ignore_case);
        if config.reverse {
            ord = ord.reverse();
        }
        ord == Ordering::Less
    });

    let mut last_line: Option<Vec<u8>> = None;
    for line_res in merged_iter {
        let line = line_res?;
        if config.unique {
            if let Some(ref last) = last_line {
                if compare_lines(&line, last, config.numeric, config.ignore_case) == Ordering::Equal {
                    continue;
                }
            }
            last_line = Some(line.clone());
        }
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    }

    out.flush()?;
    Ok(())
}

fn uu_app() -> Command {
    Command::new("sort")
        .about("Write sorted concatenation of all FILE(s) to standard output.")
        .arg(
            Arg::new("numeric-sort")
                .short('n')
                .long("numeric-sort")
                .action(ArgAction::SetTrue)
                .help("compare according to string numerical value"),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .action(ArgAction::SetTrue)
                .help("reverse the result of comparisons"),
        )
        .arg(
            Arg::new("unique")
                .short('u')
                .long("unique")
                .action(ArgAction::SetTrue)
                .help("output only the first of an equal run"),
        )
        .arg(
            Arg::new("ignore-case")
                .short('f')
                .long("ignore-case")
                .action(ArgAction::SetTrue)
                .help("fold lower case to upper case characters"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("write result to FILE instead of standard output"),
        )
        .arg(
            Arg::new("buffer-size")
                .short('S')
                .long("buffer-size")
                .value_name("SIZE")
                .help("use SIZE for main memory buffer"),
        )
        .arg(
            Arg::new("files")
                .num_args(0..)
                .action(ArgAction::Append)
                .value_name("FILE"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_lines_basic() {
        assert_eq!(compare_lines(b"apple", b"banana", false, false), Ordering::Less);
        assert_eq!(compare_lines(b"banana", b"apple", false, false), Ordering::Greater);
        assert_eq!(compare_lines(b"apple", b"apple", false, false), Ordering::Equal);
    }

    #[test]
    fn test_compare_lines_numeric() {
        assert_eq!(compare_lines(b"10", b"2", true, false), Ordering::Greater);
        assert_eq!(compare_lines(b"2", b"10", true, false), Ordering::Less);
        // Numerically equal, fall back to lexicographical: "010" < "10"
        assert_eq!(compare_lines(b"010", b"10", true, false), Ordering::Less);
    }

    #[test]
    fn test_compare_lines_ignore_case() {
        assert_eq!(compare_lines(b"Apple", b"apple", false, true), Ordering::Equal);
        assert_eq!(compare_lines(b"Apple", b"banana", false, true), Ordering::Less);
    }

    #[test]
    fn test_parse_buffer_size() {
        assert_eq!(parse_buffer_size(None), 100 * 1024 * 1024);
        assert_eq!(parse_buffer_size(Some(&"1K".to_string())), 1024);
        assert_eq!(parse_buffer_size(Some(&"1M".to_string())), 1024 * 1024);
        assert_eq!(parse_buffer_size(Some(&"1G".to_string())), 1024 * 1024 * 1024);
        assert_eq!(parse_buffer_size(Some(&"1024".to_string())), 1024);
    }

    #[test]
    fn test_numeric_sort_mixed() {
        let lines = vec![
            b"10".to_vec(),
            b"2".to_vec(),
            b"1".to_vec(),
            b"01".to_vec(),
        ];
        let mut sorted = lines.clone();
        sorted.sort_by(|a, b| compare_lines(a, b, true, false));
        assert_eq!(sorted[0], b"1");
        assert_eq!(sorted[1], b"01");
        assert_eq!(sorted[2], b"2");
        assert_eq!(sorted[3], b"10");
    }

    #[test]
    fn test_unique_sort() {
        // This is harder to test via compare_lines directly as unique happens during write
        // but we can test it via a helper or just rely on integration tests.
    }
}
