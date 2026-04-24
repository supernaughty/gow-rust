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

struct SedCommand {
    regex: Regex,
    replacement: String,
    global: bool,
    print_if_matched: bool,
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
                commands.push(parse_s_command(part)?);
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

fn parse_s_command(s: &str) -> Result<SedCommand> {
    if !s.starts_with('s') || s.len() < 4 {
        return Err(anyhow::anyhow!("unsupported or invalid sed command: '{}'. Only 's' command is supported.", s));
    }
    
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
                    if next.is_digit(10) {
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

    Ok(SedCommand {
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

    for line in content.lines() {
        let mut line_str = String::from_utf8_lossy(line).into_owned();
        let should_print = !quiet;

        for cmd in commands {
            let substitution_happened;
            let new_line = if cmd.global {
                let replaced = cmd.regex.replace_all(&line_str, &cmd.replacement);
                substitution_happened = !matches!(replaced, std::borrow::Cow::Borrowed(_));
                replaced.into_owned()
            } else {
                let replaced = cmd.regex.replace(&line_str, &cmd.replacement);
                substitution_happened = !matches!(replaced, std::borrow::Cow::Borrowed(_));
                replaced.into_owned()
            };

            line_str = new_line;
            if cmd.print_if_matched && substitution_happened {
                output.extend_from_slice(line_str.as_bytes());
                output.extend_from_slice(line_ending);
            }
        }

        if should_print {
            output.extend_from_slice(line_str.as_bytes());
            output.extend_from_slice(line_ending);
        }
    }
    Ok(output)
}
