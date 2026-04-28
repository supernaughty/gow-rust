# Phase 05: Search and Navigation - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 16 new/modified files
**Analogs found:** 15 / 16

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/gow-find/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | exact |
| `crates/gow-find/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-find/src/lib.rs` | service | file-I/O + event-driven | `crates/gow-grep/src/lib.rs` | role-match |
| `crates/gow-find/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-find/tests/find_tests.rs` | test | — | `crates/gow-grep/tests/integration.rs` | exact |
| `crates/gow-xargs/Cargo.toml` | config | — | `crates/gow-awk/Cargo.toml` | exact |
| `crates/gow-xargs/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-xargs/src/lib.rs` | service | batch + request-response | `crates/gow-grep/src/lib.rs` | partial-match |
| `crates/gow-xargs/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-xargs/tests/xargs_tests.rs` | test | — | `crates/gow-grep/tests/integration.rs` | exact |
| `crates/gow-less/Cargo.toml` | config | — | `crates/gow-grep/Cargo.toml` | role-match |
| `crates/gow-less/src/main.rs` | utility | — | `crates/gow-grep/src/main.rs` | exact |
| `crates/gow-less/src/lib.rs` | service | streaming + event-driven | `crates/gow-tail/src/lib.rs` | partial-match |
| `crates/gow-less/build.rs` | config | — | `crates/gow-grep/build.rs` | exact |
| `crates/gow-less/tests/less_tests.rs` | test | — | `crates/gow-grep/tests/integration.rs` | role-match |
| Root `Cargo.toml` | config | — | Root `Cargo.toml` (existing) | exact |

---

## Pattern Assignments

### `crates/gow-find/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml` (lines 1–36)

**Full Cargo.toml pattern:**
```toml
[package]
name = "gow-find"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU find — Windows port."

[[bin]]
name = "find"
path = "src/main.rs"

[lib]
name = "uu_find"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
walkdir = { workspace = true }
globset = { workspace = true }          # new dep — add to workspace first
bstr = { workspace = true }
windows-sys = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

**Key differences from grep analog:**
- Replace `regex`, `termcolor` with `globset`
- Add `windows-sys` (needed for `_setmode` in `-print0` mode)

---

### `crates/gow-find/src/main.rs` (utility)

**Analog:** `crates/gow-grep/src/main.rs` (lines 1–3)

**Exact copy pattern** (change `uu_grep` → `uu_find`):
```rust
fn main() {
    std::process::exit(uu_find::uumain(std::env::args_os()));
}
```

---

### `crates/gow-find/src/lib.rs` (service, file-I/O + event-driven)

**Analog:** `crates/gow-grep/src/lib.rs`

**Imports pattern** (lines 1–11 of grep analog, adapted for find):
```rust
use anyhow::Result;
use clap::Parser;
use globset::{GlobBuilder, GlobMatcher};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
```

**uumain entry point pattern** (grep lines 90–105):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("find: {}", e);
            2
        }
    }
}
```

**WalkDir traversal pattern** (grep lines 150–172 for WalkDir usage):
```rust
// Copy from grep lib.rs lines 150–172; adapt for find:
for entry in WalkDir::new(path)
    .min_depth(cli.mindepth)
    .max_depth(cli.maxdepth)
    .follow_links(false)
    .into_iter()
{
    match entry {
        Ok(e) => {
            if matches_predicates(&e, &cli)? {
                print_entry(&e, &cli);
            }
        }
        Err(err) => {
            eprintln!("find: {}", err);
        }
    }
}
```

**Clap Cli struct pattern** (grep lines 21–88 for derive macro style):
```rust
// Copy Cli struct pattern from grep lib.rs lines 21–88
// Use #[derive(Parser, Debug)] + #[command(name = "find", ...)]
// Each flag is a struct field with #[arg(short, long)] annotation
```

**-exec via std::process::Command** (no shell, from RESEARCH.md Pattern 2):
```rust
fn exec_for_entry(cmd_parts: &[String], path: &Path) -> Result<i32> {
    let args: Vec<&str> = cmd_parts[1..].iter().map(|a| {
        if a == "{}" { path.to_str().unwrap_or("") } else { a.as_str() }
    }).collect();

    let status = Command::new(&cmd_parts[0])
        .args(&args)
        .status()?;

    Ok(status.code().unwrap_or(1))
}
```

**-print0 binary mode stdout** (from RESEARCH.md Pattern 3):
```rust
#[cfg(target_os = "windows")]
fn set_stdout_binary_mode() {
    extern "C" {
        fn _setmode(fd: i32, flags: i32) -> i32;
    }
    const _O_BINARY: i32 = 0x8000;
    unsafe { _setmode(1, _O_BINARY); }  // stdout
}
// Call immediately after gow_core::init() when -print0 is active
```

**globset name matcher** (from RESEARCH.md Pattern 1 + Code Examples):
```rust
fn build_name_matcher(pattern: &str, case_insensitive: bool) -> Result<GlobMatcher> {
    let glob = GlobBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .literal_separator(false)
        .build()?;
    Ok(glob.compile_matcher())
}
// Match against basename only: entry.file_name(), NOT entry.path()
```

**Error pattern** (grep lines 100–104):
```rust
Err(e) => {
    eprintln!("find: {}", e);
    2
}
```

---

### `crates/gow-find/build.rs` (config)

**Analog:** `crates/gow-grep/build.rs` (lines 1–13)

**Exact copy — no changes needed:**
```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::Setting::Enabled),
        )
        .expect("unable to embed manifest");
    }
}
```

---

### `crates/gow-find/tests/find_tests.rs` (test)

**Analog:** `crates/gow-grep/tests/integration.rs` (lines 1–80)

**Imports and test structure pattern** (grep lines 1–5):
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;
```

**Integration test shape** (grep lines 6–28 pattern):
```rust
#[test]
fn test_find_name() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("foo.txt"), "").unwrap();
    fs::write(tmp.path().join("bar.rs"), "").unwrap();

    Command::cargo_bin("find").unwrap()
        .arg(tmp.path())
        .arg("-name").arg("*.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo.txt"))
        .stdout(predicate::str::is_match("(?m)^").unwrap().count(1));
}
```

**tempdir fixture pattern** (grep lines 19–28 for dir creation):
```rust
// Create nested dir structure for -maxdepth/-mindepth tests:
let tmp = tempdir().unwrap();
let sub = tmp.path().join("sub");
fs::create_dir(&sub).unwrap();
fs::write(sub.join("nested.txt"), "").unwrap();
```

---

### `crates/gow-xargs/Cargo.toml` (config)

**Analog:** `crates/gow-awk/Cargo.toml` (lines 1–29)

**Full Cargo.toml pattern** (awk is the minimal-dep template):
```toml
[package]
name = "gow-xargs"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU xargs — Windows port."

[[bin]]
name = "xargs"
path = "src/main.rs"

[lib]
name = "uu_xargs"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
bstr = { workspace = true }
windows-sys = { workspace = true }    # for _setmode binary mode

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

**Key differences from awk analog:**
- No `regex` needed
- Add `windows-sys` for `_setmode(0, _O_BINARY)` stdin binary mode

---

### `crates/gow-xargs/src/main.rs` (utility)

**Analog:** `crates/gow-grep/src/main.rs` (lines 1–3)

**Exact copy pattern** (change `uu_grep` → `uu_xargs`):
```rust
fn main() {
    std::process::exit(uu_xargs::uumain(std::env::args_os()));
}
```

---

### `crates/gow-xargs/src/lib.rs` (service, batch + request-response)

**Analog:** `crates/gow-grep/src/lib.rs` (entry point + error handling structure)

**Imports pattern:**
```rust
use anyhow::Result;
use clap::Parser;
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::process::Command;
```

**uumain pattern** (grep lines 90–105):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("xargs: {}", e);
            2
        }
    }
}
```

**Windows binary mode stdin** (from RESEARCH.md Pattern 3):
```rust
#[cfg(target_os = "windows")]
fn set_stdin_binary_mode() {
    extern "C" {
        fn _setmode(fd: i32, flags: i32) -> i32;
    }
    const _O_BINARY: i32 = 0x8000;
    unsafe { _setmode(0, _O_BINARY); }  // stdin
    // Use extern "C" (cdecl), NOT extern "system" (stdcall) — _setmode is CRT, not Win32
}
```

**Stdin tokenizer pattern** (from RESEARCH.md Pattern 7):
```rust
fn tokenize_stdin(stdin: impl BufRead, null_delimited: bool) -> Vec<String> {
    let delimiter = if null_delimited { b'\0' } else { b'\n' };
    let mut tokens = Vec::new();
    let mut buf = Vec::new();
    let mut reader = stdin;
    loop {
        buf.clear();
        match reader.read_until(delimiter, &mut buf) {
            Ok(0) => break,
            Ok(_) => {
                if buf.last() == Some(&delimiter) { buf.pop(); }
                if !null_delimited { if buf.last() == Some(&b'\r') { buf.pop(); } }
                if !buf.is_empty() {
                    if let Ok(s) = String::from_utf8(buf.clone()) {
                        tokens.push(s);
                    }
                }
            }
            Err(_) => break,
        }
    }
    tokens
}
```

**Command execution pattern** (from RESEARCH.md Pattern 2, adapted for batch):
```rust
fn exec_batch(cmd: &str, base_args: &[String], batch: &[String]) -> Result<i32> {
    let mut all_args: Vec<String> = base_args.to_vec();
    all_args.extend_from_slice(batch);
    let status = Command::new(cmd).args(&all_args).status()?;
    Ok(status.code().unwrap_or(1))
}

fn exec_with_replacement(cmd: &str, base_args: &[String], token: &str) -> Result<i32> {
    let replaced: Vec<String> = base_args.iter()
        .map(|a| a.replace("{}", token))  // substring replacement, not token match
        .collect();
    let status = Command::new(cmd).args(&replaced).status()?;
    Ok(status.code().unwrap_or(1))
}
```

---

### `crates/gow-xargs/build.rs` (config)

**Analog:** `crates/gow-grep/build.rs` (lines 1–13)

**Exact copy — no changes needed.** (Same build.rs as all other crates.)

---

### `crates/gow-xargs/tests/xargs_tests.rs` (test)

**Analog:** `crates/gow-grep/tests/integration.rs` (lines 1–28)

**Imports pattern** (grep lines 1–4):
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;
```

**stdin pipe test pattern** (grep lines 6–15 for write_stdin pattern):
```rust
#[test]
fn test_xargs_basic() {
    Command::cargo_bin("xargs").unwrap()
        .arg("echo")
        .write_stdin("foo\nbar\nbaz\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo"));
}
```

---

### `crates/gow-less/Cargo.toml` (config)

**Analog:** `crates/gow-grep/Cargo.toml` (lines 1–36) — grep is the best match for a multi-dep crate

**Full Cargo.toml pattern:**
```toml
[package]
name = "gow-less"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "GNU less — Windows port."

[[bin]]
name = "less"
path = "src/main.rs"

[lib]
name = "uu_less"
path = "src/lib.rs"

[dependencies]
gow-core = { path = "../gow-core" }
clap = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
regex = { workspace = true }            # for / search
crossterm = { workspace = true }        # new dep — add to workspace first
terminal_size = { workspace = true }    # viewport width
bstr = { workspace = true }

[build-dependencies]
embed-manifest = "1.5"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
tempfile = { workspace = true }
```

---

### `crates/gow-less/src/main.rs` (utility)

**Analog:** `crates/gow-grep/src/main.rs` (lines 1–3)

**Exact copy pattern** (change `uu_grep` → `uu_less`):
```rust
fn main() {
    std::process::exit(uu_less::uumain(std::env::args_os()));
}
```

---

### `crates/gow-less/src/lib.rs` (service, streaming + event-driven)

**Analog:** `crates/gow-tail/src/lib.rs` (streaming file I/O + follow mode shares the seek/BufReader pattern)

**Imports pattern** (tail lines 1–15 adapted):
```rust
use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use crossterm::tty::IsTty;
use regex::Regex;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write, stdout};
use std::panic;
use std::path::PathBuf;
use terminal_size::{terminal_size, Width};
```

**uumain + non-TTY fallback pattern** (tail lines 38–61 for uumain shape + RESEARCH.md headless strategy):
```rust
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // ... parse args ...

    // Non-interactive fallback for CI / piped usage
    if !stdout().is_tty() {
        if let Err(e) = print_to_stdout(source) {
            eprintln!("less: {e}");
            return 1;
        }
        return 0;
    }

    match run_pager(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("less: {e}");
            1
        }
    }
}
```

**RAII terminal guard + panic hook** (from RESEARCH.md Pattern 4):
```rust
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn run_pager() -> Result<()> {
    // Panic hook FIRST — before enable_raw_mode
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    event_loop()?;
    Ok(())
}
```

**crossterm event loop** (from RESEARCH.md Pattern 6):
```rust
fn event_loop(state: &mut PagerState) -> Result<()> {
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => state.scroll_up(1),
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => state.scroll_down(1),
                        (KeyCode::PageUp, _) | (KeyCode::Char('b'), _) => state.scroll_up(state.viewport_height),
                        (KeyCode::PageDown, _) | (KeyCode::Char(' '), _) => state.scroll_down(state.viewport_height),
                        (KeyCode::Char('g'), _) => state.jump_to_start(),
                        (KeyCode::Char('G'), _) => state.jump_to_end()?,
                        (KeyCode::Char('/'), _) => state.enter_search_mode(),
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
```

**Line-offset index struct** (from RESEARCH.md Pattern 5):
```rust
pub struct LineIndex {
    offsets: Vec<u64>,       // byte offset of start of each line
    reader: BufReader<File>,
    eof_reached: bool,
}

impl LineIndex {
    pub fn new(file: File) -> Self {
        Self { offsets: vec![0], reader: BufReader::new(file), eof_reached: false }
    }

    pub fn ensure_indexed_to(&mut self, line_num: usize) -> io::Result<()> {
        while self.offsets.len() <= line_num && !self.eof_reached {
            let mut buf = Vec::new();
            let n = self.reader.read_until(b'\n', &mut buf)?;
            if n == 0 { self.eof_reached = true; break; }
            let next_offset = self.offsets.last().unwrap() + n as u64;
            self.offsets.push(next_offset);
        }
        Ok(())
    }

    pub fn scan_to_end(&mut self) -> io::Result<usize> {
        while !self.eof_reached {
            self.ensure_indexed_to(self.offsets.len())?;
        }
        Ok(self.offsets.len() - 1)
    }

    pub fn seek_to_line(&mut self, line_num: usize) -> io::Result<()> {
        let offset = self.offsets[line_num];
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}
```

**Note on module layout:** RESEARCH.md recommends a separate `src/line_index.rs` module for `LineIndex`. Declare it in `lib.rs` with `mod line_index;` — same pattern used implicitly by grep for helper functions.

---

### `crates/gow-less/build.rs` (config)

**Analog:** `crates/gow-grep/build.rs` (lines 1–13)

**Exact copy — no changes needed.**

---

### `crates/gow-less/tests/less_tests.rs` (test)

**Analog:** `crates/gow-grep/tests/integration.rs` (lines 1–28)

**Imports + headless test pattern:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// less in non-TTY mode (stdout is not a terminal in CI) acts like cat
#[test]
fn test_less_pipe_mode_displays_content() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "hello\nworld\n").unwrap();

    Command::cargo_bin("less").unwrap()
        .arg(file_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}
```

**Unit test pattern for LineIndex** (grep lines 295–314 for in-module unit tests):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Unit test LineIndex without crossterm involvement
    #[test]
    fn test_line_index_builds_correctly() { ... }
}
```

---

### Root `Cargo.toml` (config — add workspace members + new deps)

**Analog:** Root `Cargo.toml` lines 1–103 (existing file)

**Members block addition** (lines 33–42 show phase comment pattern):
```toml
# Phase 5 — search and navigation (S05)
"crates/gow-find",
"crates/gow-xargs",
"crates/gow-less",
```
Insert after line 41 (`"crates/gow-awk",`), before the closing `]`.

**Workspace deps addition** (after line 94, `similar` / `diffy` block):
```toml
# Phase 5 additions (S05 search and navigation)
crossterm = "0.29"                   # terminal pager raw mode — gow-less only
globset = "0.4"                      # compiled glob matching — gow-find only
```

---

## Shared Patterns

### `gow_core::init()` — UTF-8 + VT100 initialization
**Source:** `crates/gow-core/src/lib.rs` lines 16–19
**Apply to:** `uumain()` in all three new lib.rs files — first line of every `uumain`
```rust
pub fn init() {
    encoding::setup_console_utf8();  // SetConsoleOutputCP(65001)
    color::enable_vt_mode();         // ENABLE_VIRTUAL_TERMINAL_PROCESSING
}
// Call as: gow_core::init();
```

### `gow_core::args::parse_gnu` — GNU-style arg parsing
**Source:** `crates/gow-grep/src/lib.rs` lines 93–94
**Apply to:** `uumain()` in `gow-find` and `gow-xargs` (and `gow-less` if it has options)
```rust
let matches = gow_core::args::parse_gnu(Cli::command(), args);
let cli = Cli::from_arg_matches(&matches).unwrap();
```

### `embed-manifest` build.rs — Windows manifest embedding
**Source:** `crates/gow-grep/build.rs` lines 1–13
**Apply to:** All three `build.rs` files — verbatim copy
```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::Setting::Enabled),
        )
        .expect("unable to embed manifest");
    }
}
```

### Error reporting convention — `utility: message` to stderr
**Source:** `crates/gow-grep/src/lib.rs` lines 100–104 and line 163
**Apply to:** All three lib.rs files — `eprintln!("find: {}", e)`, `eprintln!("xargs: {}", e)`, `eprintln!("less: {}", e)`
```rust
Err(e) => {
    eprintln!("grep: {}", e);  // replace "grep" with utility name
    2
}
// And for per-entry errors:
eprintln!("grep: {}: {}", e.path().display(), err);
```

### `assert_cmd` integration test structure
**Source:** `crates/gow-grep/tests/integration.rs` lines 1–28
**Apply to:** All three test files
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// Pattern: tempdir() → write fixture → Command::cargo_bin() → .arg() chain → .assert()
```

### WalkDir traversal with error handling
**Source:** `crates/gow-grep/src/lib.rs` lines 150–172
**Apply to:** `gow-find/src/lib.rs` — the core traversal loop
```rust
for entry in WalkDir::new(path).into_iter() {
    match entry {
        Ok(e) if e.file_type().is_file() => { /* process */ }
        Ok(_) => {}
        Err(err) => {
            eprintln!("grep: {}", err);
            any_error = true;
        }
    }
}
```

### Workspace Cargo.toml dep declaration style
**Source:** Root `Cargo.toml` lines 82–95
**Apply to:** Phase comment + dep comment format for new workspace entries
```toml
# Phase N additions (comment describes scope)
depname = "version"   # short description — crate-name only
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| (none) | — | — | All files have at least a role-match analog |

**Note on `gow-less/src/line_index.rs`:** This sub-module has no analog in the existing codebase. The pattern comes entirely from RESEARCH.md Pattern 5 (line-offset index). The `BufReader<File>` + `read_until` loop is the established idiom used in `gow-tail` for sequential reading, but the `Vec<u64>` offset index with `seek_to_line` is novel to this phase.

---

## Metadata

**Analog search scope:** `crates/gow-grep/`, `crates/gow-awk/`, `crates/gow-tail/`, `crates/gow-core/`, root `Cargo.toml`
**Files scanned:** 11 source files read
**Pattern extraction date:** 2026-04-28
