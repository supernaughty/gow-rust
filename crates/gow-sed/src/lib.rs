use anyhow::{Context, Result};
use bstr::ByteSlice;
use clap::Parser;
use gow_core::fs::atomic_rewrite;
use regex::{Regex, RegexBuilder};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{PathBuf};

#[derive(Parser, Debug)]
#[command(name = "sed", about = "GNU sed — Windows port.")]
struct Args {
    /// Add the script to the commands to be executed
    #[arg(short = 'e', long = "expression")]
    expressions: Vec<String>,

    /// Edit files in place (makes backup if SUFFIX supplied)
    #[arg(short = 'i', long = "in-place", value_name = "SUFFIX", num_args = 0..=1, default_missing_value = "")]
    in_place: Option<String>,

    /// Suppress automatic printing of pattern space
    #[arg(short = 'n', long = "quiet", alias = "silent")]
    quiet: bool,

    /// Script or first input file
    script_or_file: Option<String>,

    /// Input files
    files: Vec<PathBuf>,
}

/// The command to execute on a matched line.
#[derive(Debug)]
enum Cmd {
    Substitute {
        regex: Regex,
        replacement: String,
        global: bool,
        print_if_matched: bool,
    },
    Delete,
    Print,
    Quit,
}

/// A single sed address (line number, last line, or regex).
#[derive(Debug)]
enum Address {
    Line(usize),    // 1-based line number
    Last,           // '$' — last line
    Pattern(Regex), // /regex/ address
}

/// Address specification on a command: none, single, or range.
#[derive(Debug)]
enum AddrSpec {
    None,
    Single(Address),
    Range(Address, Address),
}

struct SedCommand {
    addr: AddrSpec,
    cmd: Cmd,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    if let Err(e) = run(args) {
        eprintln!("sed: {}", e);
        return 1;
    }
    0
}

fn run<I: IntoIterator<Item = OsString>>(args: I) -> Result<()> {
    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    let mut expressions = args.expressions;
    let mut files = args.files;

    if expressions.is_empty() {
        if let Some(s) = args.script_or_file {
            expressions.push(s);
        } else {
            return Err(anyhow::anyhow!("no script provided"));
        }
    } else if let Some(s) = args.script_or_file {
        files.insert(0, PathBuf::from(s));
    }

    let mut commands = Vec::new();
    for expr in expressions {
        // GNU sed supports multiple commands separated by ; or newline
        for part in expr.split(|c| c == ';' || c == '\n') {
            let part = part.trim();
            if !part.is_empty() {
                commands.push(parse_command(part)?);
            }
        }
    }

    if files.is_empty() {
        // Read from stdin
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input)?;
        let output = process_content(&input, &commands, args.quiet)?;
        io::stdout().write_all(&output)?;
    } else {
        for path in files {
            if let Some(suffix) = &args.in_place {
                // In-place editing
                if !suffix.is_empty() {
                    let mut backup_path = path.clone();
                    let mut ext = backup_path.extension().unwrap_or_default().to_os_string();
                    ext.push(suffix);
                    backup_path.set_extension(ext);
                    fs::copy(&path, &backup_path).with_context(|| format!("failed to create backup file {}", backup_path.display()))?;
                }

                atomic_rewrite(&path, |input| {
                    process_content(input, &commands, args.quiet)
                        .map_err(|e| gow_core::error::GowError::Custom(e.to_string()))
                }).with_context(|| format!("failed to edit file {} in place", path.display()))?;
            } else {
                // Normal processing
                let input = fs::read(&path).with_context(|| format!("failed to read file {}", path.display()))?;
                let output = process_content(&input, &commands, args.quiet)?;
                io::stdout().write_all(&output)?;
            }
        }
    }

    Ok(())
}

fn translate_bre_to_ere(pattern: &str) -> String {
    let mut translated = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next) = chars.peek() {
                    if next == '(' || next == ')' || next == '{' || next == '}' || next == '+' || next == '?' {
                        translated.push(chars.next().unwrap());
                    } else {
                        translated.push('\\');
                    }
                } else {
                    translated.push('\\');
                }
            }
            '(' | ')' | '{' | '}' | '+' | '?' | '|' => {
                // These are special in ERE, but literals in BRE.
                translated.push('\\');
                translated.push(c);
            }
            _ => translated.push(c),
        }
    }
    translated
}

/// Parse an address prefix from the beginning of a sed command string.
/// Returns (AddrSpec, remaining_command_str).
fn parse_address(s: &str) -> Result<(AddrSpec, &str)> {
    let s = s.trim_start();

    // No address: starts with a command letter
    if s.starts_with(|c: char| c.is_alphabetic()) {
        return Ok((AddrSpec::None, s));
    }

    // Dollar sign = last line
    if s.starts_with('$') {
        let after_dollar = s[1..].trim_start();
        if after_dollar.starts_with(',') {
            // Range: $,addr2 — unusual but handle it
            let after_comma = after_dollar[1..].trim_start();
            let (addr2, rest2) = parse_single_address(after_comma)?;
            return Ok((AddrSpec::Range(Address::Last, addr2), rest2));
        }
        return Ok((AddrSpec::Single(Address::Last), after_dollar));
    }

    // Regex address: /pattern/
    if s.starts_with('/') {
        let (addr1, rest) = parse_regex_address(s)?;
        let rest = rest.trim_start();
        if rest.starts_with(',') {
            let rest2 = rest[1..].trim_start();
            let (addr2, rest3) = parse_single_address(rest2)?;
            return Ok((AddrSpec::Range(addr1, addr2), rest3));
        }
        return Ok((AddrSpec::Single(addr1), rest));
    }

    // Line number address
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if end == 0 {
        return Ok((AddrSpec::None, s));
    }
    let line_num: usize = s[..end].parse().unwrap_or(0);
    let rest = s[end..].trim_start();

    if rest.starts_with(',') {
        let rest2 = rest[1..].trim_start();
        let (addr2, rest3) = parse_single_address(rest2)?;
        return Ok((AddrSpec::Range(Address::Line(line_num), addr2), rest3));
    }

    Ok((AddrSpec::Single(Address::Line(line_num)), rest))
}

fn parse_single_address(s: &str) -> Result<(Address, &str)> {
    let s = s.trim_start();
    if s.starts_with('$') {
        return Ok((Address::Last, &s[1..]));
    }
    if s.starts_with('/') {
        return parse_regex_address(s);
    }
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let n: usize = s[..end].parse().unwrap_or(1);
    Ok((Address::Line(n), &s[end..]))
}

fn parse_regex_address(s: &str) -> Result<(Address, &str)> {
    // s must start with '/'
    let mut chars = s[1..].chars();
    let mut pat = String::new();
    let mut escaped = false;
    let mut consumed = 1usize; // for the opening '/'
    for c in &mut chars {
        consumed += c.len_utf8();
        if escaped {
            pat.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '/' {
            break;
        } else {
            pat.push(c);
        }
    }
    let regex = Regex::new(&pat)
        .with_context(|| format!("invalid address regex: {}", pat))?;
    Ok((Address::Pattern(regex), &s[consumed..]))
}

/// Parse a single sed command (with optional address prefix).
/// Handles: s/pat/repl/flags, d, p, q, and address prefixes N, N,M, $, /regex/.
fn parse_command(s: &str) -> Result<SedCommand> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow::anyhow!("empty command"));
    }

    let (addr, rest) = parse_address(s)?;
    let rest = rest.trim_start();

    if rest.is_empty() {
        return Err(anyhow::anyhow!("missing command after address in: '{}'", s));
    }

    let cmd = match rest.chars().next().unwrap() {
        'd' => Cmd::Delete,
        'p' => Cmd::Print,
        'q' => Cmd::Quit,
        's' => {
            let sc = parse_s_command_inner(rest)?;
            sc
        }
        other => {
            return Err(anyhow::anyhow!(
                "unsupported sed command '{}' in: '{}'",
                other, s
            ))
        }
    };

    Ok(SedCommand { addr, cmd })
}

fn parse_s_command_inner(s: &str) -> Result<Cmd> {
    let mut chars = s.chars().skip(1);
    let delimiter = chars.next().ok_or_else(|| anyhow::anyhow!("missing delimiter"))?;

    let mut pattern = String::new();
    let mut replacement = String::new();
    let mut flags = String::new();

    let mut current = 0; // 0: pattern, 1: replacement, 2: flags
    let mut escaped = false;

    for c in chars {
        if escaped {
            match current {
                0 => pattern.push(c),
                1 => replacement.push(c),
                2 => flags.push(c),
                _ => {}
            }
            escaped = false;
        } else if c == '\\' {
            match current {
                0 => pattern.push(c),
                1 => replacement.push(c),
                2 => flags.push(c),
                _ => {}
            }
            escaped = true;
        } else if c == delimiter {
            current += 1;
            // No increment if it was escaped (handled above)
        } else {
            match current {
                0 => pattern.push(c),
                1 => replacement.push(c),
                2 => flags.push(c),
                _ => {}
            }
        }
    }

    if current < 2 {
        return Err(anyhow::anyhow!("invalid substitution command: '{}'", s));
    }

    let global = flags.contains('g');
    let ignore_case = flags.contains('I');
    let print_if_matched = flags.contains('p');

    let pattern = translate_bre_to_ere(&pattern);
    let regex = RegexBuilder::new(&pattern)
        .case_insensitive(ignore_case)
        .build()
        .with_context(|| format!("invalid regex: {}", pattern))?;

    // Convert sed replacement (\1, &) to regex replacement ($1, $0)
    let mut regex_replacement = String::new();
    let mut r_chars = replacement.chars().peekable();
    while let Some(c) = r_chars.next() {
        match c {
            '\\' => {
                if let Some(&next) = r_chars.peek() {
                    if next.is_ascii_digit() {
                        regex_replacement.push('$');
                        regex_replacement.push(r_chars.next().unwrap());
                    } else if next == '\\' || next == delimiter || next == '&' {
                        // Escaped special character
                        regex_replacement.push(r_chars.next().unwrap());
                    } else {
                        // Keep other escapes as they are
                        regex_replacement.push('\\');
                        regex_replacement.push(r_chars.next().unwrap());
                    }
                } else {
                    regex_replacement.push('\\');
                }
            }
            '&' => {
                regex_replacement.push_str("$0");
            }
            '$' => {
                // Escape $ for regex replacement
                regex_replacement.push_str("$$");
            }
            _ => {
                regex_replacement.push(c);
            }
        }
    }

    Ok(Cmd::Substitute {
        regex,
        replacement: regex_replacement,
        global,
        print_if_matched,
    })
}

fn process_content(content: &[u8], commands: &[SedCommand], quiet: bool) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Determine line ending to use for output
    let line_ending: &[u8] = if content.contains_str("\r\n") {
        b"\r\n"
    } else {
        b"\n"
    };

    // Collect all lines so we know the total for '$' address matching.
    let all_lines: Vec<&[u8]> = content.lines().collect();
    let total_lines = all_lines.len();

    // Track range state for each command that has an AddrSpec::Range
    // range_active[i] = true means we are inside the range for command i
    let mut range_active = vec![false; commands.len()];

    for (line_idx, line) in all_lines.iter().enumerate() {
        let line_num = line_idx + 1; // 1-based
        let mut line_str = String::from_utf8_lossy(line).into_owned();
        let should_print = !quiet;
        let mut deleted = false;
        let mut quit_after = false;

        'cmd_loop: for (cmd_idx, cmd) in commands.iter().enumerate() {
            // Determine whether this command's address matches this line
            let matches = match &cmd.addr {
                AddrSpec::None => true,
                AddrSpec::Single(addr) => match addr {
                    Address::Line(n) => line_num == *n,
                    Address::Last => line_num == total_lines,
                    Address::Pattern(re) => re.is_match(&line_str),
                },
                AddrSpec::Range(start, end) => {
                    // Enter range when start matches, stay until end matches
                    if !range_active[cmd_idx] {
                        let enters = match start {
                            Address::Line(n) => line_num == *n,
                            Address::Last => line_num == total_lines,
                            Address::Pattern(re) => re.is_match(&line_str),
                        };
                        if enters {
                            range_active[cmd_idx] = true;
                        }
                    }
                    if range_active[cmd_idx] {
                        // Check if this is the end of the range
                        let exits = match end {
                            Address::Line(n) => line_num >= *n,
                            Address::Last => line_num == total_lines,
                            Address::Pattern(re) => line_num > 1 && re.is_match(&line_str),
                        };
                        if exits {
                            range_active[cmd_idx] = false;
                        }
                        true
                    } else {
                        false
                    }
                }
            };

            if !matches {
                continue;
            }

            match &cmd.cmd {
                Cmd::Delete => {
                    deleted = true;
                    break 'cmd_loop;
                }
                Cmd::Print => {
                    output.extend_from_slice(line_str.as_bytes());
                    output.extend_from_slice(line_ending);
                }
                Cmd::Quit => {
                    quit_after = true;
                    break 'cmd_loop;
                }
                Cmd::Substitute { regex, replacement, global, print_if_matched } => {
                    let substitution_happened;
                    let new_line = if *global {
                        let replaced = regex.replace_all(&line_str, replacement.as_str());
                        substitution_happened = !matches!(replaced, std::borrow::Cow::Borrowed(_));
                        replaced.into_owned()
                    } else {
                        let replaced = regex.replace(&line_str, replacement.as_str());
                        substitution_happened = !matches!(replaced, std::borrow::Cow::Borrowed(_));
                        replaced.into_owned()
                    };

                    line_str = new_line;
                    if *print_if_matched && substitution_happened {
                        output.extend_from_slice(line_str.as_bytes());
                        output.extend_from_slice(line_ending);
                    }
                }
            }
        }

        if !deleted && should_print {
            output.extend_from_slice(line_str.as_bytes());
            output.extend_from_slice(line_ending);
        }

        if quit_after {
            break;
        }
    }

    Ok(output)
}
