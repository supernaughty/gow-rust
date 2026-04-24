use clap::Parser;
use bstr::ByteSlice;
use std::ffi::OsString;
use std::io::{self, BufRead, Write};
use std::fs::File;

#[derive(Parser)]
#[command(name = "cut", about = "GNU cut — Windows port.", version)]
struct Args {
    #[arg(short = 'b', long = "bytes", help = "select only these bytes")]
    bytes: Option<String>,

    #[arg(short = 'c', long = "characters", help = "select only these characters")]
    characters: Option<String>,

    #[arg(short = 'd', long = "delimiter", default_value = "\t", help = "use DELIM instead of TAB for field delimiter")]
    delimiter: String,

    #[arg(short = 'f', long = "fields", help = "select only these fields")]
    fields: Option<String>,

    #[arg(short = 'n', help = "(ignored)")]
    ignored_n: bool,

    #[arg(short = 's', long = "only-delimited", help = "do not print lines not containing delimiters")]
    only_delimited: bool,

    #[arg(long = "complement", help = "complement the set of selected bytes, characters or fields")]
    complement: bool,

    #[arg(long = "output-delimiter", help = "use STRING as the output delimiter")]
    output_delimiter: Option<String>,

    #[arg(help = "FILES")]
    files: Vec<String>,
}

struct Range {
    start: usize, // 1-indexed
    end: Option<usize>, // 1-indexed, inclusive
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    let mode = if let Some(ref b) = args.bytes {
        Mode::Bytes(parse_ranges(b))
    } else if let Some(ref c) = args.characters {
        Mode::Characters(parse_ranges(c))
    } else if let Some(ref f) = args.fields {
        Mode::Fields(parse_ranges(f))
    } else {
        eprintln!("cut: you must specify a list of bytes, characters, or fields");
        return 1;
    };

    let delim = if args.delimiter.len() != 1 {
        eprintln!("cut: the delimiter must be a single character");
        return 1;
    } else {
        args.delimiter.as_bytes()[0]
    };

    let out_delim = args.output_delimiter.as_deref().unwrap_or(&args.delimiter);

    let mut exit_code = 0;
    if args.files.is_empty() {
        if let Err(e) = process_input(io::stdin().lock(), &mode, delim, out_delim, &args) {
            eprintln!("cut: {}", e);
            exit_code = 1;
        }
    } else {
        for file in &args.files {
            if file == "-" {
                if let Err(e) = process_input(io::stdin().lock(), &mode, delim, out_delim, &args) {
                    eprintln!("cut: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(file) {
                    Ok(f) => {
                        if let Err(e) = process_input(io::BufReader::new(f), &mode, delim, out_delim, &args) {
                            eprintln!("cut: {}: {}", file, e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("cut: {}: {}", file, e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    exit_code
}

enum Mode {
    Bytes(Vec<Range>),
    Characters(Vec<Range>),
    Fields(Vec<Range>),
}

fn parse_ranges(s: &str) -> Vec<Range> {
    let mut ranges = Vec::new();
    for part in s.split(',') {
        if let Some(pos) = part.find('-') {
            let start_s = &part[..pos];
            let end_s = &part[pos + 1..];
            let start = if start_s.is_empty() { 1 } else { start_s.parse().unwrap_or(1) };
            let end = if end_s.is_empty() { None } else { Some(end_s.parse().unwrap_or(usize::MAX)) };
            ranges.push(Range { start, end });
        } else {
            let val = part.parse().unwrap_or(1);
            ranges.push(Range { start: val, end: Some(val) });
        }
    }
    ranges
}

fn is_selected(n: usize, ranges: &[Range], complement: bool) -> bool {
    let mut selected = false;
    for r in ranges {
        if n >= r.start && (r.end.is_none() || n <= r.end.unwrap()) {
            selected = true;
            break;
        }
    }
    if complement { !selected } else { selected }
}

fn process_input<R: BufRead>(mut input: R, mode: &Mode, delim: u8, out_delim: &str, args: &Args) -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut line = Vec::new();
    loop {
        line.clear();
        match input.read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line.ends_with(b"\n") {
                    line.pop();
                    if line.ends_with(b"\r") {
                        line.pop();
                    }
                }

                match mode {
                    Mode::Bytes(ranges) => {
                        for (i, &b) in line.iter().enumerate() {
                            if is_selected(i + 1, ranges, args.complement) {
                                stdout.write_all(&[b])?;
                            }
                        }
                        stdout.write_all(b"\n")?;
                    }
                    Mode::Characters(ranges) => {
                        for (i, ch) in line.chars().enumerate() {
                            if is_selected(i + 1, ranges, args.complement) {
                                let mut buf = [0u8; 4];
                                stdout.write_all(ch.encode_utf8(&mut buf).as_bytes())?;
                            }
                        }
                        stdout.write_all(b"\n")?;
                    }
                    Mode::Fields(ranges) => {
                        let fields: Vec<&[u8]> = line.split(|&b| b == delim).collect();
                        if fields.len() == 1 && !line.contains(&delim) {
                            if !args.only_delimited {
                                stdout.write_all(&line)?;
                                stdout.write_all(b"\n")?;
                            }
                            continue;
                        }

                        let mut first = true;
                        for (i, field) in fields.iter().enumerate() {
                            if is_selected(i + 1, ranges, args.complement) {
                                if !first {
                                    stdout.write_all(out_delim.as_bytes())?;
                                }
                                stdout.write_all(field)?;
                                first = false;
                            }
                        }
                        stdout.write_all(b"\n")?;
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ranges() {
        let r = parse_ranges("1,3-5,10-");
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].start, 1);
        assert_eq!(r[0].end, Some(1));
        assert_eq!(r[1].start, 3);
        assert_eq!(r[1].end, Some(5));
        assert_eq!(r[2].start, 10);
        assert_eq!(r[2].end, None);
    }

    #[test]
    fn test_is_selected() {
        let r = parse_ranges("1,3-5");
        assert!(is_selected(1, &r, false));
        assert!(!is_selected(2, &r, false));
        assert!(is_selected(3, &r, false));
        assert!(is_selected(4, &r, false));
        assert!(is_selected(5, &r, false));
        assert!(!is_selected(6, &r, false));

        assert!(!is_selected(1, &r, true));
        assert!(is_selected(2, &r, true));
    }
}
