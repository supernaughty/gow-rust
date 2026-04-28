//! `uu_less`: GNU less — Windows port (R017 / LESS-01).
//!
//! Implements scroll, /search, ANSI passthrough, lazy line indexing per
//! CONTEXT.md D-07 through D-10. Includes BOTH RAII terminal guard AND
//! std::panic::set_hook so the terminal is restored on every exit path.
//!
//! Headless mode: when stdout is not a TTY, behaves like `cat` so
//! integration tests via assert_cmd can drive the binary in CI.

pub mod line_index;
pub use line_index::LineIndex;

use anyhow::{anyhow, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::ResetColor;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::tty::IsTty;
use crossterm::ExecutableCommand;
use regex::Regex;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::panic;
use std::path::PathBuf;
use std::time::Duration;
use terminal_size::{terminal_size, Height, Width};

#[derive(Parser, Debug)]
#[command(
    name = "less",
    about = "GNU less — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true,
)]
struct Cli {
    /// File to page (omit to read from stdin)
    file: Option<PathBuf>,

    /// Print help information
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version information
    #[arg(long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("less: {}", e);
            return 2;
        }
    };

    // Non-TTY fallback: behave like `cat`. Critical for CI testing (RESEARCH.md headless strategy).
    if !io::stdout().is_tty() {
        return match copy_to_stdout(&cli) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("less: {}", e);
                1
            }
        };
    }

    match run_pager(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("less: {}", e);
            1
        }
    }
}

/// Non-interactive mode: stream source to stdout byte-for-byte.
/// ANSI bytes pass through unchanged (D-08 — `io::copy` is byte-faithful).
fn copy_to_stdout(cli: &Cli) -> Result<()> {
    let mut out = io::stdout().lock();
    match &cli.file {
        Some(path) => {
            let mut f =
                File::open(path).map_err(|e| anyhow!("{}: {}", path.display(), e))?;
            io::copy(&mut f, &mut out)?;
        }
        None => {
            let stdin = io::stdin();
            let mut s = stdin.lock();
            io::copy(&mut s, &mut out)?;
        }
    }
    out.flush()?;
    Ok(())
}

/// Open the source as a seekable `LineIndex`.
/// For stdin (non-seekable), buffer all content to a temp file first.
/// This mirrors GNU `less` behavior for piped input (RESEARCH.md A3).
fn open_source(cli: &Cli) -> Result<LineIndex> {
    match &cli.file {
        Some(path) => {
            let f =
                File::open(path).map_err(|e| anyhow!("{}: {}", path.display(), e))?;
            Ok(LineIndex::new(f))
        }
        None => {
            // Stdin is non-seekable; buffer to a temp file so LineIndex can seek.
            let mut tmp = tempfile::NamedTempFile::new()?;
            io::copy(&mut io::stdin().lock(), tmp.as_file_mut())?;
            tmp.as_file().sync_all()?;
            let f = File::open(tmp.path())?;
            Ok(LineIndex::new(f))
        }
    }
}

/// RAII terminal guard — restores terminal on every normal exit path,
/// including `?`-propagated errors. Paired with a panic hook for `panic!()`.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn run_pager(cli: Cli) -> Result<()> {
    // Install panic hook BEFORE enable_raw_mode (Pitfall 1 / Threat T-05-less-01).
    // Both the hook AND the RAII guard are required:
    //   - Hook handles panic!() and stack overflow.
    //   - Guard handles normal exit and ?-propagated errors.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let _guard = TerminalGuard; // drop = restore on every exit path

    let mut state = PagerState::new(open_source(&cli)?)?;
    state.render()?;
    event_loop(&mut state)?;
    Ok(())
}

struct PagerState {
    index: LineIndex,
    top_line: usize,
    viewport_h: u16,
    viewport_w: u16,
    search_pattern: Option<Regex>,
    match_lines: Vec<usize>,
    current_match_idx: Option<usize>,
}

impl PagerState {
    fn new(index: LineIndex) -> Result<Self> {
        let (cols, rows) = match terminal_size() {
            Some((Width(w), Height(h))) => (w, h),
            None => (80, 24),
        };
        Ok(Self {
            index,
            top_line: 0,
            viewport_h: rows,
            viewport_w: cols,
            search_pattern: None,
            match_lines: Vec::new(),
            current_match_idx: None,
        })
    }

    /// Number of body lines available (viewport minus status bar).
    fn body_height(&self) -> usize {
        self.viewport_h.saturating_sub(1) as usize
    }

    fn render(&mut self) -> Result<()> {
        let mut out = io::stdout().lock();
        out.execute(Clear(ClearType::All))?;
        out.execute(MoveTo(0, 0))?;
        out.execute(ResetColor)?;
        let h = self.body_height();
        for i in 0..h {
            let line_num = self.top_line + i;
            self.index.ensure_indexed_to(line_num + 1)?;
            if let Some(line) = self.index.read_line_at(line_num)? {
                // Write raw bytes — preserves ANSI escape sequences (D-08).
                out.write_all(&line)?;
                if !line.ends_with(b"\n") {
                    out.write_all(b"\n")?;
                }
            } else {
                // GNU less convention: `~` for lines past EOF.
                out.write_all(b"~\n")?;
            }
        }
        // Status line at the bottom.
        out.execute(MoveTo(0, self.viewport_h.saturating_sub(1)))?;
        let prompt = match &self.search_pattern {
            Some(p) => format!(":/{}/ (n=next, N=prev, q=quit) ", p.as_str()),
            None => ":".to_string(),
        };
        out.write_all(prompt.as_bytes())?;
        out.flush()?;
        Ok(())
    }

    fn scroll_up(&mut self, n: usize) {
        self.top_line = self.top_line.saturating_sub(n);
    }

    fn scroll_down(&mut self, n: usize) {
        // Allow scrolling forward; ensure_indexed_to in render will lazily fetch more.
        self.top_line = self.top_line.saturating_add(n);
    }

    fn jump_to_start(&mut self) {
        self.top_line = 0;
    }

    fn jump_to_end(&mut self) -> Result<()> {
        // scan_to_end is blocking on large files (Pitfall 5, A2 — documented known limitation).
        let total = self.index.scan_to_end()?;
        self.top_line = total.saturating_sub(self.body_height());
        Ok(())
    }

    fn enter_search(&mut self) -> Result<()> {
        // Collect keystrokes until Enter to build the regex pattern.
        let mut pattern = String::new();
        loop {
            let mut out = io::stdout().lock();
            out.execute(MoveTo(0, self.viewport_h.saturating_sub(1)))?;
            out.execute(Clear(ClearType::CurrentLine))?;
            write!(out, "/{}", pattern)?;
            out.flush()?;
            drop(out);

            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match (code, modifiers) {
                    (KeyCode::Enter, _) => break,
                    (KeyCode::Esc, _) => return Ok(()),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Backspace, _) => {
                        pattern.pop();
                    }
                    (KeyCode::Char(c), _) => pattern.push(c),
                    _ => {}
                }
            }
        }

        if pattern.is_empty() {
            return Ok(());
        }

        let re = Regex::new(&pattern).map_err(|e| anyhow!("bad regex: {}", e))?;
        self.search_pattern = Some(re);
        self.match_lines.clear();
        self.current_match_idx = None;
        // Scan all indexed lines (and extend forward) to find matches.
        self.find_all_matches()?;
        if let Some(&first) = self.match_lines.first() {
            self.top_line = first;
            self.current_match_idx = Some(0);
        }
        Ok(())
    }

    /// Scan all currently-indexed lines (and up to 100 000 forward) for pattern matches.
    fn find_all_matches(&mut self) -> Result<()> {
        let re = match &self.search_pattern {
            Some(r) => r.clone(),
            None => return Ok(()),
        };
        let mut line = 0usize;
        loop {
            self.index.ensure_indexed_to(line + 1)?;
            if line >= self.index.line_count_so_far() && self.index.is_complete() {
                break;
            }
            match self.index.read_line_at(line)? {
                Some(buf) => {
                    let text = std::str::from_utf8(&buf).unwrap_or("");
                    if re.is_match(text) {
                        self.match_lines.push(line);
                    }
                }
                None => break,
            }
            line += 1;
            if line > 100_000 {
                // Safety cap — gap-closure plan can remove this limit.
                break;
            }
        }
        Ok(())
    }

    fn next_match(&mut self) {
        if self.match_lines.is_empty() {
            return;
        }
        let next = match self.current_match_idx {
            Some(i) if i + 1 < self.match_lines.len() => i + 1,
            _ => 0,
        };
        self.current_match_idx = Some(next);
        self.top_line = self.match_lines[next];
    }

    fn prev_match(&mut self) {
        if self.match_lines.is_empty() {
            return;
        }
        let prev = match self.current_match_idx {
            Some(i) if i > 0 => i - 1,
            _ => self.match_lines.len() - 1,
        };
        self.current_match_idx = Some(prev);
        self.top_line = self.match_lines[prev];
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.viewport_w = w;
        self.viewport_h = h;
    }
}

fn event_loop(state: &mut PagerState) -> Result<()> {
    loop {
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                            state.scroll_up(1);
                        }
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                            state.scroll_down(1);
                        }
                        (KeyCode::PageUp, _) | (KeyCode::Char('b'), _) => {
                            let h = state.body_height();
                            state.scroll_up(h);
                        }
                        (KeyCode::PageDown, _) | (KeyCode::Char(' '), _) => {
                            let h = state.body_height();
                            state.scroll_down(h);
                        }
                        (KeyCode::Char('g'), _) => state.jump_to_start(),
                        (KeyCode::Char('G'), _) => state.jump_to_end()?,
                        (KeyCode::Char('/'), _) => state.enter_search()?,
                        (KeyCode::Char('n'), _) => state.next_match(),
                        (KeyCode::Char('N'), _) => state.prev_match(),
                        _ => {}
                    }
                }
                Event::Resize(w, h) => state.resize(w, h),
                _ => {}
            }
            state.render()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    use super::*;

    // Structural test: verify the non-TTY copy_to_stdout symbol exists and is reachable.
    // Full byte-equality is verified in the integration tests via assert_cmd (non-TTY mode).
    #[test]
    fn test_copy_to_stdout_symbol_exists() {
        // Coerce fn pointer to verify it compiles and is accessible.
        let _ = copy_to_stdout as fn(&Cli) -> Result<()>;
    }
}
