//! `uu_printf`: GNU `printf` — C-style format string evaluator.
//!
//! Does NOT use clap. Manual argv parsing (format string + positional args).
//! Extra args beyond format specifiers cause the format string to repeat.

use std::ffi::OsString;
use std::io::{self, Write};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let mut iter = args.into_iter();
    let _argv0 = iter.next();
    let all_args: Vec<String> = iter.map(|a| a.to_string_lossy().to_string()).collect();

    if all_args.is_empty() {
        eprintln!("printf: missing operand");
        return 1;
    }

    let format_str = &all_args[0];
    let mut arg_idx = 1usize;
    let total_args = all_args.len();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Repeat format string until all arguments are consumed.
    // If no % specifiers exist, print once regardless.
    loop {
        let (output, args_consumed) = format_one_pass(format_str, &all_args[arg_idx..]);
        if let Err(e) = out.write_all(output.as_bytes()) {
            eprintln!("printf: write error: {e}");
            return 1;
        }
        if args_consumed == 0 || arg_idx >= total_args {
            break;
        }
        arg_idx += args_consumed;
        if arg_idx >= total_args {
            break;
        }
    }
    0
}

/// Parse one pass through the format string, consuming args from the slice.
///
/// Returns `(output, number_of_args_consumed)`.
fn format_one_pass(fmt: &str, args: &[String]) -> (String, usize) {
    let mut output = String::new();
    let mut consumed = 0usize;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Process escape sequence
                if let Some(escaped) = process_escape(&mut chars) {
                    output.push(escaped);
                }
            }
            '%' => {
                // Parse format specifier: [flags][width][.precision][type]
                let mut flags = String::new();
                let mut width_str = String::new();
                let mut precision_str: Option<String> = None;

                // Flags: -, 0, +, space, #
                while let Some(&fc) = chars.peek() {
                    if matches!(fc, '-' | '0' | '+' | ' ' | '#') {
                        flags.push(fc);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Width: decimal digits
                while let Some(&wc) = chars.peek() {
                    if wc.is_ascii_digit() {
                        width_str.push(wc);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Precision: .digits
                if chars.peek() == Some(&'.') {
                    chars.next(); // consume '.'
                    let mut prec = String::new();
                    while let Some(&pc) = chars.peek() {
                        if pc.is_ascii_digit() {
                            prec.push(pc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    precision_str = Some(prec);
                }

                // Type character
                let type_char = match chars.next() {
                    Some(t) => t,
                    None => break,
                };

                if type_char == '%' {
                    // Literal percent — no arg consumed
                    output.push('%');
                } else {
                    // Consume one arg (empty string if out of args)
                    let arg = if consumed < args.len() {
                        args[consumed].as_str()
                    } else {
                        ""
                    };
                    consumed += 1;
                    let width = width_str.parse::<usize>().unwrap_or(0);
                    let formatted = format_spec(&flags, width, precision_str.as_deref(), type_char, arg);
                    output.push_str(&formatted);
                }
            }
            other => {
                output.push(other);
            }
        }
    }

    (output, consumed)
}

/// Process a backslash escape sequence. Called after consuming the `\`.
fn process_escape(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
    match chars.next()? {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        '\\' => Some('\\'),
        'a' => Some('\x07'),
        'b' => Some('\x08'),
        'f' => Some('\x0C'),
        'v' => Some('\x0B'),
        c @ '0'..='7' => {
            // Octal: up to 3 digits total (including c)
            let mut val = c as u32 - '0' as u32;
            for _ in 0..2 {
                if let Some(&nc) = chars.peek() {
                    if nc.is_ascii_digit() && nc <= '7' {
                        val = val * 8 + (nc as u32 - '0' as u32);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            char::from_u32(val % 256)
        }
        'x' => {
            // Hex: up to 2 digits
            let mut val = 0u32;
            let mut count = 0;
            for _ in 0..2 {
                if let Some(&hc) = chars.peek() {
                    if let Some(d) = hc.to_digit(16) {
                        val = val * 16 + d;
                        chars.next();
                        count += 1;
                    } else {
                        break;
                    }
                }
            }
            if count == 0 {
                Some('x')
            } else {
                char::from_u32(val % 256)
            }
        }
        other => Some(other), // unknown escape: pass through
    }
}

/// Format a single value according to the format spec.
fn format_spec(
    flags: &str,
    width: usize,
    precision: Option<&str>,
    type_char: char,
    arg: &str,
) -> String {
    let left_align = flags.contains('-');
    let zero_pad = flags.contains('0') && !left_align;
    let force_sign = flags.contains('+');

    match type_char {
        'd' | 'i' => {
            let val = parse_int(arg);
            let s = if force_sign && val >= 0 {
                format!("+{val}")
            } else {
                format!("{val}")
            };
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, zero_pad && val < 0)
        }
        'u' => {
            let val = arg.parse::<u64>().unwrap_or(0);
            let s = format!("{val}");
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'o' => {
            let val = parse_uint(arg);
            let s = format!("{val:o}");
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'x' => {
            let val = parse_uint(arg);
            let s = format!("{val:x}");
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'X' => {
            let val = parse_uint(arg);
            let s = format!("{val:X}");
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        's' => {
            let prec = precision.and_then(|p| p.parse::<usize>().ok());
            let s = if let Some(max_len) = prec {
                let truncated: String = arg.chars().take(max_len).collect();
                truncated
            } else {
                arg.to_string()
            };
            if width > s.len() {
                if left_align {
                    format!("{s:<width$}", s = s, width = width)
                } else {
                    format!("{s:>width$}", s = s, width = width)
                }
            } else {
                s
            }
        }
        'f' => {
            let val = parse_float(arg);
            let prec = precision.and_then(|p| p.parse::<usize>().ok()).unwrap_or(6);
            let s = if force_sign && val >= 0.0 {
                format!("+{val:.prec$}")
            } else {
                format!("{val:.prec$}")
            };
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, zero_pad && val < 0.0)
        }
        'e' => {
            let val = parse_float(arg);
            let prec = precision.and_then(|p| p.parse::<usize>().ok()).unwrap_or(6);
            let s = format_scientific(val, prec, false);
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'E' => {
            let val = parse_float(arg);
            let prec = precision.and_then(|p| p.parse::<usize>().ok()).unwrap_or(6);
            let s = format_scientific(val, prec, true);
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'g' | 'G' => {
            let val = parse_float(arg);
            let prec = precision.and_then(|p| p.parse::<usize>().ok()).unwrap_or(6);
            let prec = if prec == 0 { 1 } else { prec };
            let s = format_g(val, prec, type_char == 'G');
            pad_string(&s, width, left_align, if zero_pad { '0' } else { ' ' }, false)
        }
        'c' => {
            let ch = arg.chars().next().unwrap_or('\0');
            let s = ch.to_string();
            pad_string(&s, width, left_align, ' ', false)
        }
        other => {
            // Unknown specifier: emit as-is
            format!("%{other}")
        }
    }
}

/// Pad a string to `width` characters.
/// If `zero_pad` is true and the string starts with '-', place padding after sign.
fn pad_string(s: &str, width: usize, left_align: bool, pad_char: char, is_negative_zero_pad: bool) -> String {
    if width == 0 || s.len() >= width {
        return s.to_string();
    }
    let pad_count = width - s.len();
    if left_align {
        let mut result = s.to_string();
        for _ in 0..pad_count {
            result.push(' ');
        }
        result
    } else if is_negative_zero_pad && s.starts_with('-') {
        // Put zeros after the '-'
        let mut result = String::with_capacity(width);
        result.push('-');
        for _ in 0..pad_count {
            result.push('0');
        }
        result.push_str(&s[1..]);
        result
    } else {
        let mut result = String::with_capacity(width);
        for _ in 0..pad_count {
            result.push(pad_char);
        }
        result.push_str(s);
        result
    }
}

/// Format a float in scientific notation (e.g., 1.234567e+00).
fn format_scientific(val: f64, prec: usize, uppercase: bool) -> String {
    // Rust doesn't have direct scientific notation with fixed exponent width,
    // so we implement it manually.
    if val == 0.0 {
        let exp_char = if uppercase { 'E' } else { 'e' };
        return format!("{:.prec$}{exp_char}+00", 0.0_f64, prec = prec);
    }
    let exp = val.abs().log10().floor() as i32;
    let mantissa = val / 10f64.powi(exp);
    let exp_char = if uppercase { 'E' } else { 'e' };
    let sign = if exp >= 0 { '+' } else { '-' };
    let exp_abs = exp.unsigned_abs();
    let exp_str = if exp_abs < 10 {
        format!("{sign}0{exp_abs}")
    } else {
        format!("{sign}{exp_abs}")
    };
    format!("{mantissa:.prec$}{exp_char}{exp_str}", prec = prec)
}

/// Format a float with %g/%G semantics (shorter of %e/%f).
fn format_g(val: f64, prec: usize, uppercase: bool) -> String {
    if val == 0.0 {
        return "0".to_string();
    }
    let exp = val.abs().log10().floor() as i32;
    // Use %e if exponent < -4 or >= prec, else %f
    if exp < -4 || exp >= prec as i32 {
        // Use scientific notation with prec-1 significant digits
        let s = format_scientific(val, prec.saturating_sub(1), uppercase);
        // Remove trailing zeros from mantissa
        trim_trailing_zeros_sci(&s)
    } else {
        // Use fixed notation with prec significant digits
        let decimal_places = (prec as i32 - 1 - exp).max(0) as usize;
        let s = format!("{val:.decimal_places$}");
        trim_trailing_zeros_fixed(&s)
    }
}

fn trim_trailing_zeros_fixed(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

fn trim_trailing_zeros_sci(s: &str) -> String {
    // Split at e/E
    if let Some(pos) = s.find(|c| c == 'e' || c == 'E') {
        let (mantissa, exponent) = s.split_at(pos);
        let mantissa = trim_trailing_zeros_fixed(mantissa);
        format!("{mantissa}{exponent}")
    } else {
        trim_trailing_zeros_fixed(s)
    }
}

fn parse_int(s: &str) -> i64 {
    // Support octal (0NNN), hex (0x...), decimal
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).unwrap_or(0)
    } else if s.starts_with('0') && s.len() > 1 {
        i64::from_str_radix(&s[1..], 8).unwrap_or_else(|_| s.parse::<i64>().unwrap_or(0))
    } else {
        s.parse::<i64>().unwrap_or(0)
    }
}

fn parse_uint(s: &str) -> u64 {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).unwrap_or(0)
    } else if s.starts_with('0') && s.len() > 1 {
        u64::from_str_radix(&s[1..], 8).unwrap_or_else(|_| s.parse::<u64>().unwrap_or(0))
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
}

fn parse_float(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}
