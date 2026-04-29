use std::ffi::OsString;
use std::io::Write;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "seq",
    about = "GNU seq — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Use STRING as the separator between numbers (default: \n)
    #[arg(short = 's', long = "separator", default_value = "\n")]
    separator: String,

    /// Positional arguments: 1..=3 numbers (LAST | FIRST LAST | FIRST INC LAST)
    #[arg(allow_hyphen_values = true)]
    numbers: Vec<String>,
}

/// Count decimal places in a string representation of a number.
fn decimal_places(s: &str) -> u32 {
    if let Some(dot_pos) = s.find('.') {
        (s.len() - dot_pos - 1) as u32
    } else {
        0
    }
}

fn run(cli: &Cli) -> i32 {
    let (first_str, inc_str, last_str) = match cli.numbers.len() {
        0 => {
            eprintln!("seq: missing operand");
            eprintln!("Try 'seq --help' for more information.");
            return 2;
        }
        1 => ("1".to_string(), "1".to_string(), cli.numbers[0].clone()),
        2 => (cli.numbers[0].clone(), "1".to_string(), cli.numbers[1].clone()),
        3 => (
            cli.numbers[0].clone(),
            cli.numbers[1].clone(),
            cli.numbers[2].clone(),
        ),
        _ => {
            eprintln!("seq: extra operand '{}'", cli.numbers[3]);
            eprintln!("Try 'seq --help' for more information.");
            return 2;
        }
    };

    // Parse as f64 to validate
    let first: f64 = match first_str.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("seq: invalid floating point argument: '{first_str}'");
            return 1;
        }
    };
    let inc: f64 = match inc_str.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("seq: invalid floating point argument: '{inc_str}'");
            return 1;
        }
    };
    let last: f64 = match last_str.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("seq: invalid floating point argument: '{last_str}'");
            return 1;
        }
    };

    // Reject NaN/Infinity (T-10-02-03: mitigate non-finite values)
    if !first.is_finite() {
        eprintln!("seq: invalid floating point argument: '{first_str}'");
        return 1;
    }
    if !inc.is_finite() {
        eprintln!("seq: invalid floating point argument: '{inc_str}'");
        return 1;
    }
    if !last.is_finite() {
        eprintln!("seq: invalid floating point argument: '{last_str}'");
        return 1;
    }

    // Reject zero increment
    if inc == 0.0 {
        eprintln!("seq: invalid Zero increment value: '0'");
        return 1;
    }

    // Determine precision from input strings (not parsed floats)
    let precision = decimal_places(&first_str)
        .max(decimal_places(&inc_str))
        .max(decimal_places(&last_str));

    seq_output(first, inc, last, precision, &cli.separator)
}

fn seq_output(first: f64, inc: f64, last: f64, precision: u32, sep: &str) -> i32 {
    let scale: i64 = 10_i64.pow(precision);
    let mut cur = (first * scale as f64).round() as i64;
    let inc_scaled = (inc * scale as f64).round() as i64;
    let end = (last * scale as f64).round() as i64;
    let going_up = inc_scaled > 0;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut first_item = true;

    // Determine if separator is the default newline.
    // GNU seq always ends with a trailing newline regardless of -s.
    // When using a custom separator, values are joined by sep, then a single \n terminates.
    let use_custom_sep = sep != "\n";

    loop {
        if going_up && cur > end {
            break;
        }
        if !going_up && cur < end {
            break;
        }

        if use_custom_sep {
            if !first_item {
                let _ = out.write_all(sep.as_bytes());
            }
            first_item = false;
            if precision == 0 {
                let _ = write!(out, "{cur}");
            } else {
                let _ = write!(
                    out,
                    "{:.prec$}",
                    cur as f64 / scale as f64,
                    prec = precision as usize
                );
            }
        } else {
            // Default newline separator — each value on its own line
            let _ = if precision == 0 {
                writeln!(out, "{cur}")
            } else {
                writeln!(
                    out,
                    "{:.prec$}",
                    cur as f64 / scale as f64,
                    prec = precision as usize
                )
            };
        }

        cur += inc_scaled;
    }

    // For custom separator, end with a trailing newline
    if use_custom_sep && !first_item {
        let _ = writeln!(out);
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
            eprintln!("seq: {e}");
            return 2;
        }
    };
    run(&cli)
}
