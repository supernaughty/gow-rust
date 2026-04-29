//! `uu_od`: GNU `od` — octal/hex dump utility (U-05).
//!
//! Supports:
//! - `-t` type specifiers: o[1|2|4|8], x[1|2|4|8], d[1|2|4|8], u[1|2|4|8], c
//! - `-A` address formats: o (octal, default), x (hex), d (decimal), n (none)
//! - `-N <bytes>` byte limit
//! - Multiple file operands; stdin when none given
//!
//! Default behavior: `-t o2 -A o` (two-byte octal, octal addresses).

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use clap::{ArgAction, Parser};

// ============================================================
// CLI
// ============================================================

#[derive(Parser, Debug)]
#[command(
    name = "od",
    about = "GNU od — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Address format: o (octal, default), x (hex), d (decimal), n (none)
    #[arg(short = 'A', long = "address-radix", default_value = "o")]
    address: String,

    /// Type spec: o[1248], x[1248], d[1248], u[1248], c (default: o2)
    #[arg(short = 't', long = "format", default_value = "o2")]
    type_spec: String,

    /// Limit number of bytes read
    #[arg(short = 'N', long = "read-bytes")]
    read_bytes: Option<u64>,

    /// Files to dump (stdin if none)
    files: Vec<String>,
}

// ============================================================
// Address format
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddrFmt {
    Octal,
    Hex,
    Decimal,
    None,
}

fn parse_address_format(s: &str) -> Result<AddrFmt, String> {
    match s {
        "o" => Ok(AddrFmt::Octal),
        "x" => Ok(AddrFmt::Hex),
        "d" => Ok(AddrFmt::Decimal),
        "n" => Ok(AddrFmt::None),
        other => Err(format!("od: invalid output address radix '{other}'")),
    }
}

fn format_address(offset: u64, fmt: AddrFmt) -> String {
    match fmt {
        AddrFmt::Octal => format!("{:07o}", offset),
        AddrFmt::Hex => format!("{:07x}", offset),
        AddrFmt::Decimal => format!("{:07}", offset),
        AddrFmt::None => String::new(),
    }
}

// ============================================================
// Type specifier
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeSpec {
    Octal(usize),       // unit_size in bytes
    Hex(usize),
    SignedDec(usize),
    UnsignedDec(usize),
    Char,               // always 1 byte
}

fn parse_type_spec(s: &str) -> Result<TypeSpec, String> {
    match s {
        "o" | "o2" => Ok(TypeSpec::Octal(2)),
        "o1" => Ok(TypeSpec::Octal(1)),
        "o4" => Ok(TypeSpec::Octal(4)),
        "o8" => Ok(TypeSpec::Octal(8)),
        "x" | "x2" => Ok(TypeSpec::Hex(2)),
        "x1" => Ok(TypeSpec::Hex(1)),
        "x4" => Ok(TypeSpec::Hex(4)),
        "x8" => Ok(TypeSpec::Hex(8)),
        "d" | "d2" => Ok(TypeSpec::SignedDec(2)),
        "d1" => Ok(TypeSpec::SignedDec(1)),
        "d4" => Ok(TypeSpec::SignedDec(4)),
        "d8" => Ok(TypeSpec::SignedDec(8)),
        "u" | "u2" => Ok(TypeSpec::UnsignedDec(2)),
        "u1" => Ok(TypeSpec::UnsignedDec(1)),
        "u4" => Ok(TypeSpec::UnsignedDec(4)),
        "u8" => Ok(TypeSpec::UnsignedDec(8)),
        "c" => Ok(TypeSpec::Char),
        other => Err(format!("od: invalid type string '{other}'")),
    }
}

fn unit_size(spec: TypeSpec) -> usize {
    match spec {
        TypeSpec::Octal(n) | TypeSpec::Hex(n) | TypeSpec::SignedDec(n) | TypeSpec::UnsignedDec(n) => n,
        TypeSpec::Char => 1,
    }
}

// ============================================================
// Value formatting
// ============================================================

fn format_value(bytes: &[u8], spec: TypeSpec) -> String {
    match spec {
        TypeSpec::Octal(1) => format!("{:03o}", bytes[0]),
        TypeSpec::Octal(2) => {
            let word = u16::from_le_bytes([bytes[0], bytes[1]]);
            format!("{:06o}", word)
        }
        TypeSpec::Octal(4) => {
            let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            format!("{:011o}", word)
        }
        TypeSpec::Octal(8) => {
            let word = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            format!("{:022o}", word)
        }
        TypeSpec::Hex(1) => format!("{:02x}", bytes[0]),
        TypeSpec::Hex(2) => {
            let word = u16::from_le_bytes([bytes[0], bytes[1]]);
            format!("{:04x}", word)
        }
        TypeSpec::Hex(4) => {
            let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            format!("{:08x}", word)
        }
        TypeSpec::Hex(8) => {
            let word = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            format!("{:016x}", word)
        }
        TypeSpec::SignedDec(1) => format!("{:4}", bytes[0] as i8),
        TypeSpec::SignedDec(2) => {
            let word = i16::from_le_bytes([bytes[0], bytes[1]]);
            format!("{:6}", word)
        }
        TypeSpec::SignedDec(4) => {
            let word = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            format!("{:11}", word)
        }
        TypeSpec::SignedDec(8) => {
            let word = i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            format!("{:20}", word)
        }
        TypeSpec::UnsignedDec(1) => format!("{:3}", bytes[0]),
        TypeSpec::UnsignedDec(2) => {
            let word = u16::from_le_bytes([bytes[0], bytes[1]]);
            format!("{:5}", word)
        }
        TypeSpec::UnsignedDec(4) => {
            let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            format!("{:10}", word)
        }
        TypeSpec::UnsignedDec(8) => {
            let word = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            format!("{:20}", word)
        }
        TypeSpec::Char => format_char(bytes[0]),
        // Fallback for unknown sizes — shouldn't happen with validated parse
        _ => format!("{:?}", bytes),
    }
}

/// Format a single byte as a character field (3-char wide, right-aligned).
/// GNU od -t c uses 3-char value fields; the leading separator space makes 4 visible cols.
/// Named escapes (2-char: `\n`, `\t`, etc.) right-padded to 3 → 1 leading space + 2 chars.
/// Printable ASCII (1-char) → 2 leading spaces + char.
/// Octal non-printable → 3-char octal (`001`) → fits exactly.
fn format_char(b: u8) -> String {
    match b {
        0x00 => " \\0".to_string(),
        0x07 => " \\a".to_string(),
        0x08 => " \\b".to_string(),
        0x09 => " \\t".to_string(),
        0x0A => " \\n".to_string(),
        0x0B => " \\v".to_string(),
        0x0C => " \\f".to_string(),
        0x0D => " \\r".to_string(),
        b if (0x20..=0x7E).contains(&b) => format!("{:>3}", b as char),
        // Non-printable, non-control: 3-digit octal
        b => format!("{:03o}", b),
    }
}

// ============================================================
// Dump engine
// ============================================================

/// Print the od dump for `data` with the given address and type spec.
fn dump<W: io::Write>(
    writer: &mut W,
    data: &[u8],
    addr_fmt: AddrFmt,
    spec: TypeSpec,
) -> io::Result<()> {
    let bytes_per_line: usize = 16;
    let unit = unit_size(spec);

    let mut offset: u64 = 0;

    // Handle empty input: still emit "0000000\n"
    if data.is_empty() {
        let addr = format_address(0, addr_fmt);
        writeln!(writer, "{addr}")?;
        return Ok(());
    }

    for chunk in data.chunks(bytes_per_line) {
        let addr = format_address(offset, addr_fmt);
        write!(writer, "{addr}")?;

        // Iterate over unit-sized sub-chunks within this row
        // For partial rows (< bytes_per_line), just process the bytes available
        // For partial last unit (< unit bytes), process byte-by-byte in fallback
        let mut i = 0;
        while i < chunk.len() {
            let remaining = chunk.len() - i;
            if remaining >= unit {
                let val = format_value(&chunk[i..i + unit], spec);
                write!(writer, " {val}")?;
                i += unit;
            } else {
                // Partial unit at end of data: emit individual bytes using unit-1 fallback
                // GNU od pads the partial word with zeros and prints it
                // We emit each remaining byte padded into a full unit
                let mut padded = vec![0u8; unit];
                padded[..remaining].copy_from_slice(&chunk[i..i + remaining]);
                let val = format_value(&padded, spec);
                write!(writer, " {val}")?;
                i += remaining;
            }
        }

        writeln!(writer)?;
        offset += chunk.len() as u64;
    }

    // Final line: total byte count as address, no data
    let final_addr = format_address(offset, addr_fmt);
    writeln!(writer, "{final_addr}")?;

    Ok(())
}

// ============================================================
// I/O helpers
// ============================================================

fn read_input(files: &[String], limit: Option<u64>) -> (Vec<u8>, i32) {
    let mut buf: Vec<u8> = Vec::new();
    let mut exit_code = 0;

    if files.is_empty() {
        let stdin = io::stdin();
        let reader: Box<dyn Read> = Box::new(stdin.lock());
        if let Err(e) = read_limited(reader, &mut buf, limit) {
            eprintln!("od: stdin: {e}");
            exit_code = 1;
        }
    } else {
        for path_str in files {
            let converted = gow_core::path::try_convert_msys_path(path_str);
            let path = Path::new(&converted);
            match File::open(path) {
                Ok(f) => {
                    let remaining = limit.map(|lim| lim.saturating_sub(buf.len() as u64));
                    if let Err(e) = read_limited(f, &mut buf, remaining) {
                        eprintln!("od: {converted}: {e}");
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("od: {converted}: {e}");
                    exit_code = 1;
                }
            }
            // If we've hit the limit, stop reading more files
            if let Some(lim) = limit {
                if buf.len() as u64 >= lim {
                    break;
                }
            }
        }
    }

    (buf, exit_code)
}

fn read_limited<R: Read>(mut reader: R, buf: &mut Vec<u8>, limit: Option<u64>) -> io::Result<()> {
    match limit {
        None => {
            reader.read_to_end(buf)?;
        }
        Some(n) => {
            let already = buf.len() as u64;
            let to_read = n.saturating_sub(already);
            if to_read == 0 {
                return Ok(());
            }
            let mut tmp = vec![0u8; to_read as usize];
            let mut read_so_far = 0usize;
            loop {
                if read_so_far >= to_read as usize {
                    break;
                }
                match reader.read(&mut tmp[read_so_far..]) {
                    Ok(0) => break,
                    Ok(n) => read_so_far += n,
                    Err(e) => return Err(e),
                }
            }
            buf.extend_from_slice(&tmp[..read_so_far]);
        }
    }
    Ok(())
}

// ============================================================
// Entry point
// ============================================================

fn run(cli: Cli) -> i32 {
    let addr_fmt = match parse_address_format(&cli.address) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };

    let spec = match parse_type_spec(&cli.type_spec) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };

    let (buf, read_exit) = read_input(&cli.files, cli.read_bytes);

    // Even on read errors, we dump what we got (GNU od behavior for mixed errors)
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if let Err(e) = dump(&mut out, &buf, addr_fmt, spec) {
        eprintln!("od: write error: {e}");
        return 1;
    }

    read_exit
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let cli = Cli::parse_from(args);
    run(cli)
}

// ============================================================
// Unit tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_addr_fmt_all_variants() {
        assert_eq!(parse_address_format("o").unwrap(), AddrFmt::Octal);
        assert_eq!(parse_address_format("x").unwrap(), AddrFmt::Hex);
        assert_eq!(parse_address_format("d").unwrap(), AddrFmt::Decimal);
        assert_eq!(parse_address_format("n").unwrap(), AddrFmt::None);
        assert!(parse_address_format("q").is_err());
    }

    #[test]
    fn parse_type_spec_defaults() {
        assert_eq!(parse_type_spec("o").unwrap(), TypeSpec::Octal(2));
        assert_eq!(parse_type_spec("o2").unwrap(), TypeSpec::Octal(2));
        assert_eq!(parse_type_spec("x").unwrap(), TypeSpec::Hex(2));
        assert_eq!(parse_type_spec("c").unwrap(), TypeSpec::Char);
        assert!(parse_type_spec("z").is_err());
    }

    #[test]
    fn format_address_octal() {
        assert_eq!(format_address(0, AddrFmt::Octal), "0000000");
        assert_eq!(format_address(8, AddrFmt::Octal), "0000010");
        assert_eq!(format_address(16, AddrFmt::Octal), "0000020");
    }

    #[test]
    fn format_address_hex() {
        assert_eq!(format_address(16, AddrFmt::Hex), "0000010");
    }

    #[test]
    fn format_address_decimal() {
        assert_eq!(format_address(16, AddrFmt::Decimal), "0000016");
    }

    #[test]
    fn format_address_none() {
        assert_eq!(format_address(0, AddrFmt::None), "");
        assert_eq!(format_address(16, AddrFmt::None), "");
    }

    #[test]
    fn format_char_printable() {
        assert_eq!(format_char(b'A'), "  A");
        assert_eq!(format_char(b' '), "   ");
    }

    #[test]
    fn format_char_escape_sequences() {
        assert_eq!(format_char(0x0A), " \\n");
        assert_eq!(format_char(0x09), " \\t");
        assert_eq!(format_char(0x0D), " \\r");
        assert_eq!(format_char(0x00), " \\0");
    }

    #[test]
    fn format_char_nonprintable_octal() {
        // 0x01 = 001
        assert_eq!(format_char(0x01), "001");
    }

    #[test]
    fn unit_sizes() {
        assert_eq!(unit_size(TypeSpec::Octal(2)), 2);
        assert_eq!(unit_size(TypeSpec::Hex(1)), 1);
        assert_eq!(unit_size(TypeSpec::Char), 1);
    }

    #[test]
    fn le_word_reading() {
        // "ab" = 0x61 0x62; LE u16 = 0x6261 = 25153 decimal = 061141 octal
        let bytes = [0x61u8, 0x62u8];
        let word = u16::from_le_bytes([bytes[0], bytes[1]]);
        assert_eq!(format!("{:06o}", word), "061141");
    }

    #[test]
    fn dump_empty_input() {
        let mut out = Vec::new();
        dump(&mut out, &[], AddrFmt::Octal, TypeSpec::Octal(2)).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "0000000\n");
    }

    #[test]
    fn dump_two_bytes_o2() {
        let mut out = Vec::new();
        dump(&mut out, b"ab", AddrFmt::Octal, TypeSpec::Octal(2)).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "0000000 061141\n0000002\n");
    }

    #[test]
    fn dump_four_bytes_o2() {
        let mut out = Vec::new();
        dump(&mut out, b"abcd", AddrFmt::Octal, TypeSpec::Octal(2)).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "0000000 061141 062143\n0000004\n");
    }
}
