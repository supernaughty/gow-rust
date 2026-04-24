use anyhow::Result;
use bstr::ByteSlice;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, ValueEnum};
use regex::bytes::{Regex, RegexBuilder};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};
use walkdir::WalkDir;

#[derive(ValueEnum, Clone, Debug, Default, PartialEq)]
enum ColorArg {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Parser, Debug)]
#[command(
    name = "grep",
    about = "GNU grep — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// The pattern to search for
    #[arg(required_unless_present_any = ["help", "version"])]
    pattern: Option<String>,

    /// The files or directories to search
    #[arg(default_value = "-")]
    files: Vec<PathBuf>,

    /// Ignore case distinctions in patterns and input data
    #[arg(short, long)]
    ignore_case: bool,

    /// Invert the sense of matching, to select non-matching lines
    #[arg(short = 'v', long)]
    invert_match: bool,

    /// Read all files under each directory, recursively
    #[arg(short, long, short_alias = 'R')]
    recursive: bool,

    /// Prefix each line of output with the 1-based line number within its input file
    #[arg(short = 'n', long)]
    line_number: bool,

    /// Suppress normal output; instead print the name of each input file from which output would normally have been printed
    #[arg(short = 'l', long = "files-with-matches")]
    files_with_matches: bool,

    /// Suppress normal output; instead print a count of matching lines for each input file
    #[arg(short, long)]
    count: bool,

    /// Suppress the prefixing of file names on output
    #[arg(short = 'h', long = "no-filename")]
    no_filename: bool,

    /// Print the file name for each match
    #[arg(short = 'H', long = "with-filename")]
    with_filename: bool,

    /// Interpret PATTERNS as extended regular expressions (ERE)
    #[arg(short = 'E', long)]
    extended_regexp: bool,

    /// Interpret PATTERNS as fixed strings, not regular expressions
    #[arg(short = 'F', long)]
    fixed_strings: bool,

    /// Use markers to highlight the matching strings
    #[arg(long, default_value = "auto")]
    color: ColorArg,

    /// Print help information
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Print version information
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok((true, _)) => 0,
        Ok((false, false)) => 1,
        Ok((false, true)) => 2,
        Err(e) => {
            eprintln!("grep: {}", e);
            2
        }
    }
}

fn run(cli: Cli) -> Result<(bool, bool)> {
    let pattern_str = match &cli.pattern {
        Some(p) => p,
        None => return Ok((false, false)),
    };

    let pattern = if cli.fixed_strings {
        regex::escape(pattern_str)
    } else {
        pattern_str.clone()
    };

    let regex = RegexBuilder::new(&pattern)
        .case_insensitive(cli.ignore_case)
        .build()?;

    let color_choice = match cli.color {
        ColorArg::Always => ColorChoice::Always,
        ColorArg::Never => ColorChoice::Never,
        ColorArg::Auto => ColorChoice::Auto,
    };
    let mut stdout = gow_core::color::stdout(color_choice);

    let multiple_files = cli.files.len() > 1 || cli.recursive;
    let show_filename = (multiple_files && !cli.no_filename) || cli.with_filename;

    let mut any_match = false;
    let mut any_error = false;
    for path in &cli.files {
        if path == Path::new("-") {
            match search_stdin(&regex, &cli, &mut stdout) {
                Ok(matched) => {
                    if matched {
                        any_match = true;
                    }
                }
                Err(e) => {
                    eprintln!("grep: (standard input): {}", e);
                    any_error = true;
                }
            }
        } else if path.is_dir() {
            if cli.recursive {
                for entry in WalkDir::new(path).into_iter() {
                    match entry {
                        Ok(e) if e.file_type().is_file() => {
                            match search_file(e.path(), &regex, &cli, &mut stdout, show_filename) {
                                Ok(matched) => {
                                    if matched {
                                        any_match = true;
                                    }
                                }
                                Err(err) => {
                                    eprintln!("grep: {}: {}", e.path().display(), err);
                                    any_error = true;
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("grep: {}", err);
                            any_error = true;
                        }
                    }
                }
            } else {
                eprintln!("grep: {}: Is a directory", path.display());
                any_error = true;
            }
        } else {
            match search_file(path, &regex, &cli, &mut stdout, show_filename) {
                Ok(matched) => {
                    if matched {
                        any_match = true;
                    }
                }
                Err(e) => {
                    eprintln!("grep: {}: {}", path.display(), e);
                    any_error = true;
                }
            }
        }
    }

    Ok((any_match, any_error))
}

fn search_stdin(regex: &Regex, cli: &Cli, stdout: &mut impl WriteColor) -> Result<bool> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    process_reader(reader, "(standard input)", regex, cli, stdout, false)
}

fn search_file(
    path: &Path,
    regex: &Regex,
    cli: &Cli,
    stdout: &mut impl WriteColor,
    show_filename: bool,
) -> Result<bool> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    process_reader(
        reader,
        &path.to_string_lossy(),
        regex,
        cli,
        stdout,
        show_filename,
    )
}

fn process_reader<R: BufRead>(
    mut reader: R,
    filename: &str,
    regex: &Regex,
    cli: &Cli,
    stdout: &mut impl WriteColor,
    show_filename: bool,
) -> Result<bool> {
    let mut line_num = 0;
    let mut match_count = 0;
    let mut buf = Vec::new();
    let mut reader_any_match = false;

    loop {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        line_num += 1;

        let line = &buf;
        let is_match = regex.is_match(line);
        let should_print = if cli.invert_match { !is_match } else { is_match };

        if should_print {
            match_count += 1;
            reader_any_match = true;
            if cli.files_with_matches {
                writeln!(stdout, "{}", filename)?;
                return Ok(true);
            }
            if !cli.count {
                if show_filename {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
                    write!(stdout, "{}:", filename)?;
                    stdout.reset()?;
                }
                if cli.line_number {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                    write!(stdout, "{}:", line_num)?;
                    stdout.reset()?;
                }

                if !cli.invert_match && stdout.supports_color() {
                    let mut last = 0;
                    for m in regex.find_iter(line) {
                        write!(stdout, "{}", line[last..m.start()].as_bstr())?;
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                        write!(stdout, "{}", line[m.start()..m.end()].as_bstr())?;
                        stdout.reset()?;
                        last = m.end();
                    }
                    write!(stdout, "{}", line[last..].as_bstr())?;
                } else {
                    write!(stdout, "{}", line.as_bstr())?;
                }
                if !line.ends_with(b"\n") {
                    writeln!(stdout)?;
                }
            }
        }
    }

    if cli.count {
        if show_filename {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
            write!(stdout, "{}:", filename)?;
            stdout.reset()?;
        }
        writeln!(stdout, "{}", match_count)?;
    }

    Ok(reader_any_match)
}

#[cfg(test)]
mod tests {
    use super::*;
    use termcolor::Buffer;

    #[test]
    fn test_basic_match() -> Result<()> {
        let regex = Regex::new("world")?;
        let cli = Cli::try_parse_from(["grep", "world"])?;
        let mut buf = Buffer::no_color();

        let input = b"hello\nworld\nrust\n";
        process_reader(&input[..], "test", &regex, &cli, &mut buf, false)?;

        assert_eq!(buf.into_inner(), b"world\n");
        Ok(())
    }

    #[test]
    fn test_invert_match() -> Result<()> {
        let regex = Regex::new("world")?;
        let cli = Cli::try_parse_from(["grep", "-v", "world"])?;
        let mut buf = Buffer::no_color();

        let input = b"hello\nworld\nrust\n";
        process_reader(&input[..], "test", &regex, &cli, &mut buf, false)?;

        assert_eq!(buf.into_inner(), b"hello\nrust\n");
        Ok(())
    }

    #[test]
    fn test_line_number() -> Result<()> {
        let regex = Regex::new("world")?;
        let cli = Cli::try_parse_from(["grep", "-n", "world"])?;
        let mut buf = Buffer::no_color();

        let input = b"hello\nworld\nrust\n";
        process_reader(&input[..], "test", &regex, &cli, &mut buf, false)?;

        assert_eq!(buf.into_inner(), b"2:world\n");
        Ok(())
    }
}
