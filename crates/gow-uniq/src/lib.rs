use clap::Parser;
use std::ffi::OsString;
use std::io::{self, BufRead, Write, BufReader};
use std::fs::File;

#[derive(Parser)]
#[command(name = "uniq", about = "GNU uniq — Windows port.", version)]
struct Args {
    #[arg(short = 'c', long = "count", help = "prefix lines by the number of occurrences")]
    count: bool,

    #[arg(short = 'd', long = "repeated", help = "only print duplicate lines, one for each group")]
    repeated: bool,

    #[arg(short = 'D', help = "print all duplicate lines")]
    all_repeated_short: bool,

    #[arg(long = "all-repeated", help = "print all duplicate lines; METHOD={none(default),prepend,separate}")]
    all_repeated: Option<Option<String>>,

    #[arg(short = 'f', long = "skip-fields", default_value = "0", help = "avoid comparing the first N fields")]
    skip_fields: usize,

    #[arg(short = 'i', long = "ignore-case", help = "ignore differences in case when comparing")]
    ignore_case: bool,

    #[arg(short = 's', long = "skip-chars", default_value = "0", help = "avoid comparing the first N characters")]
    skip_chars: usize,

    #[arg(short = 'u', long = "unique", help = "only print unique lines")]
    unique: bool,

    #[arg(short = 'w', long = "check-chars", help = "compare no more than N characters in lines")]
    check_chars: Option<usize>,

    #[arg(short = 'z', long = "zero-terminated", help = "line delimiter is NUL, not newline")]
    zero_terminated: bool,

    #[arg(help = "INPUT")]
    input: Option<String>,

    #[arg(help = "OUTPUT")]
    output: Option<String>,
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

    let mut input_reader: Box<dyn BufRead> = if let Some(ref input_path) = args.input {
        if input_path == "-" {
            Box::new(io::stdin().lock())
        } else {
            match File::open(input_path) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => {
                    eprintln!("uniq: {}: {}", input_path, e);
                    return 1;
                }
            }
        }
    } else {
        Box::new(io::stdin().lock())
    };

    let mut output_writer: Box<dyn Write> = if let Some(ref output_path) = args.output {
        match File::create(output_path) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("uniq: {}: {}", output_path, e);
                return 1;
            }
        }
    } else {
        Box::new(io::stdout().lock())
    };

    if let Err(e) = process_uniq(&mut *input_reader, &mut *output_writer, &args) {
        eprintln!("uniq: {}", e);
        return 1;
    }

    0
}

fn process_uniq(input: &mut dyn BufRead, output: &mut dyn Write, args: &Args) -> io::Result<()> {
    let delim = if args.zero_terminated { b'\0' } else { b'\n' };
    let mut line = Vec::new();
    let mut prev_line: Option<Vec<u8>> = None;
    let mut count = 0;

    loop {
        line.clear();
        match input.read_until(delim, &mut line) {
            Ok(0) => {
                if let Some(prev) = prev_line {
                    write_line(output, &prev, count, args)?;
                }
                break;
            }
            Ok(_) => {
                if prev_line.is_none() {
                    prev_line = Some(line.clone());
                    count = 1;
                } else {
                    let prev = prev_line.as_ref().unwrap();
                    if compare_lines(prev, &line, args) {
                        count += 1;
                    } else {
                        write_line(output, prev, count, args)?;
                        prev_line = Some(line.clone());
                        count = 1;
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn compare_lines(l1: &[u8], l2: &[u8], args: &Args) -> bool {
    let s1 = get_compare_part(l1, args);
    let s2 = get_compare_part(l2, args);

    if args.ignore_case {
        if s1.len() != s2.len() {
            return false;
        }
        for (b1, b2) in s1.iter().zip(s2.iter()) {
            if b1.to_ascii_lowercase() != b2.to_ascii_lowercase() {
                return false;
            }
        }
        true
    } else {
        s1 == s2
    }
}

fn get_compare_part<'a>(line: &'a [u8], args: &Args) -> &'a [u8] {
    let mut s = line;
    if s.ends_with(&[b'\n']) {
        s = &s[..s.len() - 1];
        if s.ends_with(&[b'\r']) {
            s = &s[..s.len() - 1];
        }
    } else if s.ends_with(&[b'\0']) {
        s = &s[..s.len() - 1];
    }

    // Skip fields
    if args.skip_fields > 0 {
        let mut fields_skipped = 0;
        let mut i = 0;
        while i < s.len() && fields_skipped < args.skip_fields {
            // Skip leading whitespace of field
            while i < s.len() && (s[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= s.len() {
                break;
            }
            // Skip the field itself
            while i < s.len() && !(s[i] as char).is_whitespace() {
                i += 1;
            }
            fields_skipped += 1;
        }
        s = &s[i..];
    }

    // Skip chars
    if args.skip_chars > 0 {
        let skip = std::cmp::min(args.skip_chars, s.len());
        s = &s[skip..];
    }

    // Check chars
    if let Some(n) = args.check_chars {
        let len = std::cmp::min(n, s.len());
        s = &s[..len];
    }

    s
}

fn write_line(output: &mut dyn Write, line: &[u8], count: usize, args: &Args) -> io::Result<()> {
    let print = if args.unique {
        count == 1
    } else if args.repeated {
        count > 1
    } else {
        true
    };

    if print {
        if args.count {
            write!(output, "{:>7} ", count)?;
        }
        output.write_all(line)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_compare_part() {
        let args = Args::try_parse_from(["uniq"]).unwrap();
        assert_eq!(get_compare_part(b"abc\n", &args), b"abc");
        
        let args_skip_f = Args::try_parse_from(["uniq", "-f", "1"]).unwrap();
        assert_eq!(get_compare_part(b"foo bar\n", &args_skip_f), b" bar");

        let args_skip_c = Args::try_parse_from(["uniq", "-s", "2"]).unwrap();
        assert_eq!(get_compare_part(b"abcde\n", &args_skip_c), b"cde");
    }

    #[test]
    fn test_compare_lines() {
        let args = Args::try_parse_from(["uniq"]).unwrap();
        assert!(compare_lines(b"abc\n", b"abc\r\n", &args));
        assert!(!compare_lines(b"abc\n", b"abd\n", &args));

        let args_i = Args::try_parse_from(["uniq", "-i"]).unwrap();
        assert!(compare_lines(b"abc\n", b"ABC\n", &args_i));
    }
}
