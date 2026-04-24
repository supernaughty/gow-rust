//! `uu_tail`: GNU tail — output last N lines and follow files with notify (TEXT-02).
//!
//! Covers: TEXT-02

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

const DEFAULT_LINES: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Lines,
    Bytes,
}

#[derive(Debug, Clone)]
struct Options {
    mode: Mode,
    count: u64,
    plus: bool,
    follow: bool,
    quiet: bool,
    verbose: bool,
    zero_terminated: bool,
    retry: bool,
    sleep_interval: Duration,
    pid: Option<u32>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = match setup_args().try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            e.print().unwrap();
            return 2;
        }
    };

    let options = parse_options(&matches);
    let files: Vec<PathBuf> = matches
        .get_many::<OsString>("files")
        .map(|f| f.map(PathBuf::from).collect())
        .unwrap_or_else(|| vec![PathBuf::from("-")]);

    if let Err(e) = run(&options, &files) {
        eprintln!("tail: {e:?}");
        return 1;
    }

    0
}

fn setup_args() -> Command {
    Command::new("tail")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Output the last part of files")
        .allow_negative_numbers(true) // For -n -5
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("K")
                .help("output the last K lines, instead of the last 10; or use -n +K to output starting with line K")
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("K")
                .help("output the last K bytes; or use -c +K to output starting with byte K")
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::new("follow")
                .short('f')
                .long("follow")
                .action(ArgAction::SetTrue)
                .help("output appended data as the file grows"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .alias("silent")
                .action(ArgAction::SetTrue)
                .help("never output headers giving file names"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("always output headers giving file names"),
        )
        .arg(
            Arg::new("zero-terminated")
                .short('z')
                .long("zero-terminated")
                .action(ArgAction::SetTrue)
                .help("line delimiter is NUL, not newline"),
        )
        .arg(
            Arg::new("retry")
                .long("retry")
                .action(ArgAction::SetTrue)
                .help("keep trying to open a file if it is inaccessible"),
        )
        .arg(
            Arg::new("sleep-interval")
                .short('s')
                .long("sleep-interval")
                .value_name("N")
                .help("with -f, sleep for approximately N seconds (default 1.0) between iterations"),
        )
        .arg(
            Arg::new("pid")
                .long("pid")
                .value_name("PID")
                .help("with -f, terminate after process ID, PID dies"),
        )
        .arg(
            Arg::new("files")
                .action(ArgAction::Append)
                .value_parser(clap::builder::OsStringValueParser::new())
                .num_args(0..),
        )
}

fn parse_options(matches: &clap::ArgMatches) -> Options {
    let mut mode = Mode::Lines;
    let mut count = DEFAULT_LINES;
    let mut plus = false;

    if let Some(val) = matches.get_one::<String>("bytes") {
        mode = Mode::Bytes;
        if let Some(rest) = val.strip_prefix('+') {
            plus = true;
            count = rest.parse().unwrap_or(DEFAULT_LINES);
        } else {
            count = val.parse::<i64>().map(|n| n.abs() as u64).unwrap_or(DEFAULT_LINES);
        }
    } else if let Some(val) = matches.get_one::<String>("lines") {
        mode = Mode::Lines;
        if let Some(rest) = val.strip_prefix('+') {
            plus = true;
            count = rest.parse().unwrap_or(DEFAULT_LINES);
        } else {
            count = val.parse::<i64>().map(|n| n.abs() as u64).unwrap_or(DEFAULT_LINES);
        }
    }

    Options {
        mode,
        count,
        plus,
        follow: matches.get_flag("follow"),
        quiet: matches.get_flag("quiet"),
        verbose: matches.get_flag("verbose"),
        zero_terminated: matches.get_flag("zero-terminated"),
        retry: matches.get_flag("retry"),
        sleep_interval: matches
            .get_one::<String>("sleep-interval")
            .and_then(|s| s.parse::<f64>().ok())
            .map(Duration::from_secs_f64)
            .unwrap_or(Duration::from_secs(1)),
        pid: matches
            .get_one::<String>("pid")
            .and_then(|p| p.parse::<u32>().ok()),
    }
}

fn run(options: &Options, files: &[PathBuf]) -> Result<()> {
    let multiple = files.len() > 1;
    let mut first = true;

    for path in files {
        if multiple && !options.quiet || options.verbose {
            if !first {
                println!();
            }
            if path == Path::new("-") {
                println!("==> standard input <==");
            } else {
                println!("==> {} <==", path.display());
            }
        }
        first = false;

        if let Err(e) = tail_file(options, path) {
            if options.retry && options.follow {
                eprintln!("tail: {e:?}");
            } else {
                return Err(e);
            }
        }
    }

    if options.follow {
        follow_files(options, files)?;
    }

    Ok(())
}

fn tail_file(options: &Options, path: &Path) -> Result<()> {
    if path == Path::new("-") {
        return tail_stdin(options);
    }

    let mut file = File::open(path).with_context(|| format!("cannot open '{}' for reading", path.display()))?;
    let metadata = file.metadata()?;

    if options.plus {
        let offset = if options.mode == Mode::Bytes {
            options.count.saturating_sub(1)
        } else {
            // Lines mode +N: skip N-1 lines
            skip_lines(&mut file, options.count.saturating_sub(1), options.zero_terminated)?
        };
        file.seek(SeekFrom::Start(offset))?;
        io::copy(&mut file, &mut io::stdout())?;
    } else if options.mode == Mode::Bytes {
        let size = metadata.len();
        let start = size.saturating_sub(options.count);
        file.seek(SeekFrom::Start(start))?;
        io::copy(&mut file, &mut io::stdout())?;
    } else {
        // Lines mode -N: output last N lines
        tail_lines(&mut file, options.count, options.zero_terminated)?;
    }

    Ok(())
}

fn tail_stdin(options: &Options) -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    if options.plus {
        if options.mode == Mode::Bytes {
            let mut buffer = [0u8; 8192];
            let mut skipped = 0;
            let target = options.count.saturating_sub(1);
            while skipped < target {
                let to_read = (target - skipped).min(buffer.len() as u64) as usize;
                let n = reader.read(&mut buffer[..to_read])?;
                if n == 0 { break; }
                skipped += n as u64;
            }
            io::copy(&mut reader, &mut io::stdout())?;
        } else {
            let mut line_count = 0;
            let target = options.count.saturating_sub(1);
            let delim = if options.zero_terminated { 0 } else { b'\n' };
            while line_count < target {
                let mut buf = Vec::new();
                let n = reader.read_until(delim, &mut buf)?;
                if n == 0 { break; }
                line_count += 1;
            }
            io::copy(&mut reader, &mut io::stdout())?;
        }
    } else {
        // -N mode on stdin requires buffering because we can't seek
        if options.mode == Mode::Bytes {
            let mut buffer = std::collections::VecDeque::with_capacity(options.count as usize);
            let mut chunk = [0u8; 8192];
            while let Ok(n) = reader.read(&mut chunk) {
                if n == 0 { break; }
                for &b in &chunk[..n] {
                    if buffer.len() == options.count as usize {
                        buffer.pop_front();
                    }
                    buffer.push_back(b);
                }
            }
            let (a, b) = buffer.as_slices();
            io::stdout().write_all(a)?;
            io::stdout().write_all(b)?;
        } else {
            let mut buffer = std::collections::VecDeque::with_capacity(options.count as usize);
            let delim = if options.zero_terminated { 0 } else { b'\n' };
            loop {
                let mut line = Vec::new();
                let n = reader.read_until(delim, &mut line)?;
                if n == 0 { break; }
                if buffer.len() == options.count as usize {
                    buffer.pop_front();
                }
                buffer.push_back(line);
            }
            for line in buffer {
                io::stdout().write_all(&line)?;
            }
        }
    }

    Ok(())
}

fn skip_lines(file: &mut File, count: u64, zero_terminated: bool) -> Result<u64> {
    let mut reader = BufReader::new(file);
    let mut current_offset = 0;
    let mut lines_skipped = 0;
    let delim = if zero_terminated { 0 } else { b'\n' };

    while lines_skipped < count {
        let mut buf = Vec::new();
        let n = reader.read_until(delim, &mut buf)?;
        if n == 0 { break; }
        current_offset += n as u64;
        lines_skipped += 1;
    }

    Ok(current_offset)
}

fn tail_lines(file: &mut File, count: u64, zero_terminated: bool) -> Result<()> {
    if count == 0 { return Ok(()); }

    let size = file.metadata()?.len();
    if size == 0 { return Ok(()); }

    let delim = if zero_terminated { 0 } else { b'\n' };
    
    // Better implementation of tail_lines using Seek
    let mut lines_found = 0;
    let mut pos = size;
    
    let chunk_size = 8192;
    let mut buffer = vec![0u8; chunk_size];
    
    while pos > 0 && lines_found <= count {
        let to_read = pos.min(chunk_size as u64);
        pos -= to_read;
        file.seek(SeekFrom::Start(pos))?;
        file.read_exact(&mut buffer[..to_read as usize])?;
        
        for i in (0..to_read as usize).rev() {
            if buffer[i] == delim {
                // GNU tail -n 1 on "a\n" gives "a\n".
                // If the last character is a newline, it marks the end of the last line.
                // We only start counting lines from the second-to-last newline if the last byte is a newline.
                
                if pos + i as u64 == size - 1 {
                    // Ignore trailing newline for the purpose of finding the N-th line start
                    continue;
                }
                
                lines_found += 1;
                
                if lines_found == count {
                    file.seek(SeekFrom::Start(pos + i as u64 + 1))?;
                    io::copy(file, &mut io::stdout())?;
                    return Ok(());
                }
            }
        }
    }
    
    // If we reached the beginning of the file without finding enough delimiters
    file.seek(SeekFrom::Start(0))?;
    io::copy(file, &mut io::stdout())?;

    Ok(())
}

fn follow_files(options: &Options, files: &[PathBuf]) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let mut watched_paths = Vec::new();
    for path in files {
        if path == Path::new("-") { continue; }
        let abs_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else {
            std::env::current_dir()?.join(path)
        };
        
        if let Some(parent) = abs_path.parent() {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
            watched_paths.push(abs_path);
        }
    }

    let mut file_states: std::collections::HashMap<PathBuf, u64> = std::collections::HashMap::new();
    for path in &watched_paths {
        if let Ok(meta) = std::fs::metadata(path) {
            file_states.insert(path.clone(), meta.len());
        } else if options.retry {
            // Keep it in states with 0 or a special value if it doesn't exist
            // but we want to handle it when it's created.
            // HashMap entries will be used to filter events.
            file_states.insert(path.clone(), 0);
        }
    }

    loop {
        // Check PID if provided
        if let Some(pid) = options.pid {
            if !is_process_alive(pid) {
                break;
            }
        }

        match rx.recv_timeout(options.sleep_interval) {
            Ok(Ok(event)) => {
                handle_event(options, event, &mut file_states)?;
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Periodically check if files have changed anyway (polling fallback or just to process PID check)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

fn handle_event(_options: &Options, event: Event, states: &mut std::collections::HashMap<PathBuf, u64>) -> Result<()> {
    for event_path in event.paths {
        // Try to find a match in states. We might need to normalize both.
        let mut matched_path = None;
        for watched_path in states.keys() {
            if is_same_path(&event_path, watched_path) {
                matched_path = Some(watched_path.clone());
                break;
            }
        }

        if let Some(abs_path) = matched_path {
            let old_size = states.get_mut(&abs_path).unwrap();
            if let Ok(meta) = std::fs::metadata(&abs_path) {
                let new_size = meta.len();
                if new_size > *old_size {
                    let mut file = File::open(&abs_path)?;
                    file.seek(SeekFrom::Start(*old_size))?;
                    io::copy(&mut file, &mut io::stdout())?;
                    io::stdout().flush()?;
                    *old_size = new_size;
                } else if new_size < *old_size {
                    eprintln!("tail: {}: file truncated", abs_path.display());
                    let mut file = File::open(&abs_path)?;
                    io::copy(&mut file, &mut io::stdout())?;
                    io::stdout().flush()?;
                    *old_size = new_size;
                }
            }
        }
    }
    Ok(())
}

fn is_same_path(p1: &Path, p2: &Path) -> bool {
    if p1 == p2 {
        return true;
    }
    let p1_can = p1.canonicalize().ok();
    let p2_can = p2.canonicalize().ok();
    match (p1_can, p2_can) {
        (Some(c1), Some(c2)) => c1 == c2,
        _ => {
            // Fallback to comparing components or just file names if one side doesn't exist
            p1.file_name() == p2.file_name() && p1.parent() == p2.parent()
        }
    }
}

#[cfg(target_os = "windows")]
fn is_process_alive(pid: u32) -> bool {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Threading::GetExitCodeProcess;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == std::ptr::null_mut() || handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut exit_code = 0u32;
        let success = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        
        const STILL_ACTIVE: u32 = 259;
        success != 0 && exit_code == STILL_ACTIVE
    }
}

#[cfg(not(target_os = "windows"))]
fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_args() {
        let cmd = setup_args();
        assert_eq!(cmd.get_name(), "tail");
    }
}
