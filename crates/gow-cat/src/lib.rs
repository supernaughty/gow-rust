//! `uu_cat`: GNU `cat` — concatenate files and write raw bytes to stdout (FILE-01).
//!
//! Encoding policy (D-48): raw bytes, no UTF-8 decode. Line boundaries are
//! `b'\n'` only. UTF-8/CP949 mixed content passes through unchanged.
//!
//! Flags: -n (number all), -b (number non-blank), -s (squeeze blanks),
//! -v (visualize non-print), -E (dollar before newline), -T (tab as ^I),
//! -A (shorthand for -vET).

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use clap::{Arg, ArgAction, Command};

fn uu_app() -> Command {
    Command::new("cat")
        .about("GNU cat — concatenate files and write to stdout")
        .disable_help_flag(false)
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .action(ArgAction::SetTrue)
                .help("Number all output lines"),
        )
        .arg(
            Arg::new("number-nonblank")
                .short('b')
                .long("number-nonblank")
                .action(ArgAction::SetTrue)
                .help("Number non-blank lines (overrides -n)"),
        )
        .arg(
            Arg::new("squeeze")
                .short('s')
                .long("squeeze-blank")
                .action(ArgAction::SetTrue)
                .help("Suppress repeated blank lines"),
        )
        .arg(
            Arg::new("show-nonprinting")
                .short('v')
                .long("show-nonprinting")
                .action(ArgAction::SetTrue)
                .help("Use ^ and M- notation for non-printing bytes"),
        )
        .arg(
            Arg::new("show-ends")
                .short('E')
                .long("show-ends")
                .action(ArgAction::SetTrue)
                .help("Append $ at end of each line"),
        )
        .arg(
            Arg::new("show-tabs")
                .short('T')
                .long("show-tabs")
                .action(ArgAction::SetTrue)
                .help("Display TAB characters as ^I"),
        )
        .arg(
            Arg::new("show-all")
                .short('A')
                .long("show-all")
                .action(ArgAction::SetTrue)
                .help("Equivalent to -vET"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

#[derive(Debug, Clone, Copy, Default)]
struct Opts {
    number: bool,
    number_nonblank: bool,
    squeeze: bool,
    show_nonprinting: bool,
    show_ends: bool,
    show_tabs: bool,
}

impl Opts {
    fn from_matches(m: &clap::ArgMatches) -> Self {
        let show_all = m.get_flag("show-all");
        let mut o = Opts {
            number: m.get_flag("number"),
            number_nonblank: m.get_flag("number-nonblank"),
            squeeze: m.get_flag("squeeze"),
            show_nonprinting: m.get_flag("show-nonprinting") || show_all,
            show_ends: m.get_flag("show-ends") || show_all,
            show_tabs: m.get_flag("show-tabs") || show_all,
        };
        // -b overrides -n (GNU behavior)
        if o.number_nonblank {
            o.number = false;
        }
        o
    }
}

/// Encode a single byte per GNU cat -v/-E/-T rules.
/// Appends the encoded form to `out`. Must be called for every byte in the
/// input stream — handles newline ($ suffix) and tab (^I) inline.
fn visualize_byte(b: u8, out: &mut Vec<u8>, opts: Opts) {
    match b {
        b'\n' => {
            if opts.show_ends {
                out.push(b'$');
            }
            out.push(b'\n');
        }
        b'\t' => {
            if opts.show_tabs {
                out.extend_from_slice(b"^I");
            } else {
                out.push(b'\t');
            }
        }
        0x00..=0x1f => {
            if opts.show_nonprinting {
                out.push(b'^');
                out.push(b ^ 0x40);
            } else {
                out.push(b);
            }
        }
        0x7f => {
            if opts.show_nonprinting {
                out.extend_from_slice(b"^?");
            } else {
                out.push(b);
            }
        }
        0x80..=0xff => {
            if opts.show_nonprinting {
                out.extend_from_slice(b"M-");
                visualize_byte(b & 0x7f, out, opts);
            } else {
                out.push(b);
            }
        }
        _ => out.push(b),
    }
}

struct CatState {
    line_number: u64, // only incremented when a line is actually numbered
    prev_blank: bool, // for -s squeeze
}

fn is_blank_line(bytes: &[u8]) -> bool {
    // Blank = empty OR only \n OR only whitespace + \n
    bytes
        .iter()
        .all(|&b| b == b'\n' || b == b'\r' || b == b' ' || b == b'\t')
}

fn cat_reader<R: Read, W: Write>(
    reader: R,
    writer: &mut W,
    opts: Opts,
    state: &mut CatState,
) -> io::Result<()> {
    let mut reader = BufReader::new(reader);
    let mut raw = Vec::with_capacity(8192);
    let mut out = Vec::with_capacity(8192);

    loop {
        raw.clear();
        let n = reader.read_until(b'\n', &mut raw)?;
        if n == 0 {
            break;
        }

        let blank = is_blank_line(&raw);

        // -s squeeze: collapse runs of blank lines into one
        if opts.squeeze && blank && state.prev_blank {
            continue;
        }
        state.prev_blank = blank;

        // Decide whether this line gets a number
        let numbered = opts.number || (opts.number_nonblank && !blank);

        out.clear();
        if numbered {
            state.line_number += 1;
            // GNU format: 6-space right-aligned + tab
            let prefix = format!("{:>6}\t", state.line_number);
            out.extend_from_slice(prefix.as_bytes());
        }

        for &b in &raw {
            visualize_byte(b, &mut out, opts);
        }

        writer.write_all(&out)?;
    }
    Ok(())
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let opts = Opts::from_matches(&matches);

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    let mut stdout = io::stdout().lock();
    let mut state = CatState {
        line_number: 0,
        prev_blank: false,
    };
    let mut exit_code = 0;

    if operands.is_empty() {
        if let Err(e) = cat_reader(io::stdin().lock(), &mut stdout, opts, &mut state) {
            eprintln!("cat: -: {e}");
            exit_code = 1;
        }
        return exit_code;
    }

    for op in &operands {
        if op == "-" {
            if let Err(e) = cat_reader(io::stdin().lock(), &mut stdout, opts, &mut state) {
                eprintln!("cat: -: {e}");
                exit_code = 1;
            }
            continue;
        }
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);
        match File::open(path) {
            Ok(f) => {
                if let Err(e) = cat_reader(f, &mut stdout, opts, &mut state) {
                    eprintln!("cat: {converted}: {e}");
                    exit_code = 1;
                }
            }
            Err(e) => {
                eprintln!("cat: {converted}: {e}");
                exit_code = 1;
                // Continue — GNU processes all operands
            }
        }
    }
    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vz(b: u8, opts: Opts) -> Vec<u8> {
        let mut out = Vec::new();
        visualize_byte(b, &mut out, opts);
        out
    }

    fn opts_v() -> Opts {
        Opts {
            show_nonprinting: true,
            ..Opts::default()
        }
    }
    fn opts_e() -> Opts {
        Opts {
            show_ends: true,
            ..Opts::default()
        }
    }
    fn opts_t() -> Opts {
        Opts {
            show_tabs: true,
            ..Opts::default()
        }
    }

    #[test]
    fn visualize_cr_under_v() {
        assert_eq!(vz(b'\r', opts_v()), b"^M");
    }

    #[test]
    fn visualize_del_under_v() {
        assert_eq!(vz(0x7f, opts_v()), b"^?");
    }

    #[test]
    fn visualize_highbit_under_v() {
        assert_eq!(vz(0x82, opts_v()), b"M-^B");
    }

    #[test]
    fn visualize_newline_with_e() {
        assert_eq!(vz(b'\n', opts_e()), b"$\n");
    }

    #[test]
    fn visualize_tab_with_t() {
        assert_eq!(vz(b'\t', opts_t()), b"^I");
    }

    #[test]
    fn visualize_passthrough_letter() {
        assert_eq!(vz(b'A', Opts::default()), b"A");
    }

    #[test]
    fn visualize_newline_no_flag_is_plain() {
        assert_eq!(vz(b'\n', Opts::default()), b"\n");
    }

    #[test]
    fn visualize_tab_no_flag_is_plain() {
        assert_eq!(vz(b'\t', Opts::default()), b"\t");
    }

    #[test]
    fn is_blank_empty() {
        assert!(is_blank_line(b""));
    }

    #[test]
    fn is_blank_only_newline() {
        assert!(is_blank_line(b"\n"));
    }

    #[test]
    fn is_blank_only_whitespace() {
        assert!(is_blank_line(b"  \t\n"));
    }

    #[test]
    fn is_blank_with_letter() {
        assert!(!is_blank_line(b"a\n"));
    }

    #[test]
    fn opts_b_overrides_n() {
        // Simulate ArgMatches by constructing opts manually through from_matches path:
        // -b forces number=false
        let mut cmd = uu_app();
        let m = cmd
            .try_get_matches_from_mut(vec!["cat", "-n", "-b"])
            .unwrap();
        let o = Opts::from_matches(&m);
        assert!(o.number_nonblank);
        assert!(!o.number);
    }

    #[test]
    fn opts_a_shorthand_enables_vet() {
        let mut cmd = uu_app();
        let m = cmd.try_get_matches_from_mut(vec!["cat", "-A"]).unwrap();
        let o = Opts::from_matches(&m);
        assert!(o.show_nonprinting);
        assert!(o.show_ends);
        assert!(o.show_tabs);
    }
}
