# Phase 05: Search and Navigation - Research

**Researched:** 2026-04-28
**Domain:** GNU navigation utilities — find (file traversal + predicates), xargs (stdin command builder), less (interactive terminal pager)
**Confidence:** HIGH (codebase verified, key APIs confirmed via docs.rs and official sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**find: Name Matching**
- D-01: `-name` is case-sensitive (strict GNU behavior). Add `-iname` for case-insensitive matching — do not make `-name` Windows-aware.
- D-02: Wildcard support uses standard POSIX glob: `*`, `?`, `[...]`. Use `globset` crate (add to workspace deps). No `**` recursive glob in `-name` — that is not GNU find behavior.

**find: Predicate Set**
- D-03: Implement all four predicate groups: `-type f/d/l` (file/dir/symlink), `-size +N/-N` (with bytes/k/M/G units), `-mtime/-atime/-ctime` (days), `-maxdepth/-mindepth` (depth control).
- D-04: `-atime` note — Windows does not track access time by default; implement the flag but document that atime may equal mtime on NTFS without the `NtfsDisableLastAccessUpdate` registry change.

**find: -exec Behavior**
- D-05: Support `-exec cmd {} \;` form only. The arg-accumulating `{} +` form and `-execdir` are deferred to gap-closure plans.
- D-06: Execute commands via `std::process::Command` (CreateProcess) — no shell intermediary. This handles paths with spaces natively (fixes GOW #209) and avoids cmd.exe quoting issues.

**less: Feature Depth**
- D-07: Core pager feature set: arrow keys + PgUp/PgDn scroll, `q` quit, `/` forward search with `n`/`N` navigation, `G`/`g` jump to end/start. File arguments and stdin piping both supported.
- D-08: ANSI color passthrough enabled by default (like `less -R`). Detect ANSI escape sequences in input and render them — `grep --color | less` shows highlighted output.
- D-09: Streaming/buffered I/O — never load the full file into memory. Read forward lazily, buffer a sliding window. `G` (jump to end) requires a seek but must not OOM on large files.
- D-10: Add `crossterm` to workspace deps. Use it for raw terminal mode, cursor movement, and screen clearing. `termcolor` is not sufficient for a full interactive pager.

**xargs: Scope**
- D-11: Serial-only execution — no `-P N` parallel mode in this phase. Defer parallel xargs to a gap-closure plan.
- D-12: Core flags to implement: `-0` (null-separated input, pairs with `find -print0`), `-I {}` (fixed `{}` replacement string only — no configurable `-I STR`), `-n maxargs`, `-L maxlines`.

### Claude's Discretion
- Internal buffering strategy for less (ring buffer vs. line index)
- globset vs. manual glob matching for find (globset is the right choice given it's already in the tech stack)
- Integration test structure (follow established patterns from phases 3-4)
- Whether to add `-print0` to find (needed for xargs -0 interop — yes, include it)

### Deferred Ideas (OUT OF SCOPE)
- `find -exec cmd {} +` (arg-accumulating form) — gap-closure plan after Phase 05
- `find -execdir` — gap-closure plan after Phase 05
- `xargs -P N` (parallel execution) — gap-closure plan after Phase 05
- `xargs -I STR` (configurable replacement string) — gap-closure plan after Phase 05
- `less` line numbers (`:N` toggle), marks (`m`/`M`), `LESS` env var, `-e` auto-exit — Phase 05 gap-closure or Phase 07+
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| R015 | File search: `-name`, `-type`, `-size`, `-mtime` predicates; `-exec` support; space handling in paths | walkdir 2.5 for traversal; globset 0.4 for glob matching; `std::process::Command` for `-exec`; `std::fs::Metadata` for size/time predicates |
| R016 | stdin command builder: `-0` null-separated input, `-I {}` replacement, `-n`, `-L` | `BufRead::read_until(b'\0')` for null tokenization; Windows binary mode stdin via `_setmode`; `std::process::Command` for invocation |
| R017 | File pager: scroll, `/` search, large file support without OOM | crossterm 0.29 raw mode + alternate screen; `File::seek` for G-jump; line-offset index `Vec<u64>` for navigation; `regex` for search highlighting |
</phase_requirements>

---

## Summary

Phase 05 implements three independent binary crates (`gow-find`, `gow-xargs`, `gow-less`) following the established `lib.rs` + `main.rs` + `build.rs` pattern verified in every Phase 2–4 crate. The codebase already has `walkdir` and `regex` as workspace dependencies. Two new workspace dependencies are required: `crossterm = "0.29"` (for `less`) and `globset = "0.4"` (for `find -name`/`-iname`).

The three biggest Windows-specific engineering challenges are: (1) raw mode panic safety for `less` — crossterm does not auto-restore terminal state on panic, requiring an explicit RAII guard and a `std::panic::set_hook`; (2) null-byte pipeline for `find -print0 | xargs -0` — Windows stdin/stdout in text mode silently transforms bytes, requiring `_setmode(0/1, _O_BINARY)` before reading/writing; (3) `less` G-jump to end-of-file on large files — the line-offset index approach (store `Vec<u64>` of byte offsets as lines are scanned) enables O(1) seek without loading the whole file.

`find -exec` using `std::process::Command` (CreateProcess) is the correct approach for paths with spaces — it passes each argument as a separate string to the Win32 `CreateProcessW` API, avoiding cmd.exe quoting entirely. This is consistent with D-06 and directly fixes GOW issue #209.

**Primary recommendation:** Implement `gow-find` first (walkdir + globset pattern is well-understood from `gow-ls`/`gow-grep`), then `gow-xargs` (stdin tokenizer + Command), then `gow-less` (crossterm, most complex due to raw mode lifecycle management).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Directory traversal (find) | Binary (gow-find) | gow-core path conversion | walkdir iterates, predicates filter before yielding |
| Glob name matching (find -name) | Binary (gow-find) | — | GlobBuilder builds per-invocation; case-sensitivity controlled per flag |
| File metadata predicates (-type/-size/-mtime) | Binary (gow-find) | OS (NTFS metadata) | `std::fs::Metadata` fields; NTFS atime caveat documented |
| Process spawning (find -exec) | Binary (gow-find) | OS (CreateProcessW) | std::process::Command; no shell intermediary |
| Null-byte pipeline (xargs -0) | Binary (gow-xargs) + Binary (gow-find) | OS (pipe handles) | Both ends must set binary mode; `read_until(b'\0')` tokenizer |
| Stdin tokenization (xargs) | Binary (gow-xargs) | — | BufRead::read_until for both newline and null modes |
| Interactive terminal I/O (less) | Binary (gow-less) | OS (Console API via crossterm) | crossterm raw mode + alternate screen for keyboard capture |
| Line-offset index (less navigation) | Binary (gow-less) | — | Vec<u64> index built during forward scan; seek for G-jump |
| ANSI passthrough (less -R) | Binary (gow-less) | — | Detect ESC sequences in input, pass through to output without stripping |

---

## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `walkdir` | 2.5.0 | Recursive directory traversal for find | Already used by ls, grep, cp, rm, chmod — zero setup cost [VERIFIED: Cargo.toml] |
| `globset` | 0.4.18 | Compiled glob matching for `-name`/`-iname` | BurntSushi; handles `*`, `?`, `[...]`; case_insensitive() API; by same author as regex/ripgrep [VERIFIED: crates.io] |
| `regex` | 1.12.3 | Pattern matching for `less /` search | Already workspace dep; reuse for search highlighting [VERIFIED: Cargo.toml] |
| `bstr` | 1.9.1 | Byte-safe iteration | Already workspace dep; needed for non-UTF-8 filenames in find output [VERIFIED: Cargo.toml] |
| `clap` | 4.6.1 | Argument parsing | Workspace dep; derive macro for all three utilities [VERIFIED: Cargo.toml] |
| `anyhow`/`thiserror` | 1.0 / 2.0 | Error handling | Workspace deps; same pattern as all prior crates [VERIFIED: Cargo.toml] |
| `terminal_size` | 0.4.0 | Terminal width for less viewport | Already workspace dep (used by ls) [VERIFIED: Cargo.toml] |

### New Workspace Dependencies Required
| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `crossterm` | 0.29.0 | Raw mode, keyboard events, cursor, alternate screen for `less` | D-10 locked decision; Windows Console API support; 73M+ downloads [VERIFIED: crates.io 2026-04-28] |
| `globset` | 0.4.18 | Glob compilation for find `-name`/`-iname` | D-02 locked decision; already referenced in CLAUDE.md stack table [VERIFIED: crates.io 2026-04-28] |

**Note:** `crossterm` and `globset` are NOT currently in the workspace `Cargo.toml`. The first plan task must add them. [VERIFIED: `cargo metadata` shows neither present]

### Supporting (dev dependencies, already workspace)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `assert_cmd` | 2.2.1 | Integration tests | All three crates — spawn binary, assert output |
| `predicates` | 3.1.4 | Assertion matchers | Pair with assert_cmd for contains/regex assertions |
| `tempfile` | 3.27.0 | Temp dirs for test fixtures | find traversal tests, xargs file-creation tests |
| `snapbox` | 1.2.1 | Snapshot testing | Optional — less output tests if needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `globset` | Manual `regex` glob-to-regex | globset handles `[...]` character classes and Windows path separators correctly; manual conversion is error-prone |
| `crossterm` (for less) | `termion` | termion is Unix-only; crossterm is the correct Windows choice per CLAUDE.md |
| `std::process::Command` (find -exec) | shell via `cmd /C` | Shell intermediary introduces quoting complexity; CreateProcess with per-argument strings is cleaner and fixes GOW #209 |

**Installation (add to workspace root Cargo.toml):**
```toml
crossterm = "0.29"
globset = "0.4"
```

**Version verification:** [VERIFIED: crates.io 2026-04-28]
- `crossterm = "0.29.0"` — current stable
- `globset = "0.4.18"` — current stable

---

## Architecture Patterns

### System Architecture Diagram

```
                    gow-find
                    =========
  CLI args ──► [Clap parser]
                    │
              ┌─────▼──────┐
              │ WalkDir    │◄── min_depth/max_depth (filter_entry)
              │ iterator   │
              └─────┬──────┘
                    │ DirEntry stream
              ┌─────▼──────────────────┐
              │ Predicate chain         │
              │  -type f/d/l           │◄── Metadata::file_type()
              │  -name (GlobMatcher)   │◄── GlobBuilder::case_insensitive()
              │  -size +N/-N           │◄── Metadata::len()
              │  -mtime/-ctime/-atime  │◄── Metadata::modified()/created()
              └─────┬──────────────────┘
                    │ matching paths
            ┌───────┴───────┐
            │               │
     [-print0 output]   [-exec cmd {} \;]
     null-delimited      std::process::Command
     to stdout           (CreateProcessW, no shell)

                    gow-xargs
                    =========
  stdin ──► [binary mode stdin (_setmode)]
                    │
              ┌─────▼──────┐
              │ Tokenizer  │◄── read_until(b'\0') if -0
              │            │◄── read_line() if default
              └─────┬──────┘
                    │ token batches
              ┌─────▼──────┐
              │ Batcher    │◄── -n maxargs limit
              │            │◄── -L maxlines limit
              └─────┬──────┘
                    │
              [-I {} replacement] or [append to args]
                    │
              std::process::Command::new(cmd)
                   .args(accumulated_args)
                   .status()

                    gow-less
                    =========
  file/stdin ──► [LineIndexer]
                    │ builds Vec<u64> byte offsets lazily
              ┌─────▼──────┐
              │ ViewState  │ top_line, search_pattern, viewport_height
              └─────┬──────┘
                    │
              [crossterm setup]
              enable_raw_mode()
              execute!(EnterAlternateScreen)
                    │
              ┌─────▼──────┐
              │ Render loop │ MoveTo(0,0) → write visible lines
              │             │ ANSI passthrough: write raw bytes
              └─────┬──────┘
                    │
              [event::read()]
              KeyCode::Up/Down/PageUp/PageDown → scroll
              KeyCode::Char('/')  → search mode
              KeyCode::Char('n')/'N' → next/prev match
              KeyCode::Char('G')  → seek to end (File::seek)
              KeyCode::Char('g')  → seek to start
              KeyCode::Char('q')  → cleanup + exit
                    │
              [RAII guard / panic hook]
              disable_raw_mode()
              execute!(LeaveAlternateScreen)
```

### Recommended Project Structure
```
crates/
├── gow-find/
│   ├── src/
│   │   ├── main.rs       # fn main() { std::process::exit(uu_find::uumain(args_os())); }
│   │   └── lib.rs        # uumain(), Cli struct, run(), predicate logic
│   ├── tests/
│   │   └── integration.rs
│   └── Cargo.toml
├── gow-xargs/
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs        # uumain(), tokenize_stdin(), exec_batch()
│   ├── tests/
│   │   └── integration.rs
│   └── Cargo.toml
└── gow-less/
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs        # uumain(), Pager struct, render(), event_loop()
    │   └── line_index.rs # LineIndex: Vec<u64> offsets, scan_forward(), seek_to_line()
    ├── tests/
    │   └── integration.rs  # non-interactive tests only (headless)
    └── Cargo.toml
```

### Pattern 1: walkdir + GlobBuilder for find -name / -iname

```rust
// Source: docs.rs/globset/0.4.18 + docs.rs/walkdir/2.5.0
use globset::{Glob, GlobBuilder, GlobMatcher};
use walkdir::WalkDir;

fn build_name_matcher(pattern: &str, case_insensitive: bool) -> Result<GlobMatcher> {
    let glob = GlobBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .literal_separator(false) // POSIX: * matches across path separators in -name context
        .build()?;
    Ok(glob.compile_matcher())
}

// Usage: match only the filename component (GNU find -name matches basename only)
for entry in WalkDir::new(root).min_depth(min).max_depth(max) {
    let entry = entry?;
    let file_name = entry.file_name().to_string_lossy();
    if matcher.is_match(file_name.as_ref()) {
        println!("{}", entry.path().display());
    }
}
```

**Key detail:** GNU `find -name` matches against the file's *basename only*, not the full path. Use `entry.file_name()`, not `entry.path()`.

### Pattern 2: find -exec via std::process::Command (no shell)

```rust
// Source: docs.rs/std/process::Command — Windows CreateProcess path
// Fixes GOW #209 (paths with spaces)
use std::process::Command;

fn exec_for_entry(cmd_parts: &[String], path: &Path) -> Result<i32> {
    // Replace {} with the actual path — each as a separate arg (no quoting needed)
    let args: Vec<&str> = cmd_parts[1..].iter().map(|a| {
        if a == "{}" { path.to_str().unwrap_or("") } else { a.as_str() }
    }).collect();
    
    let status = Command::new(&cmd_parts[0])
        .args(&args)
        .status()?;
    
    Ok(status.code().unwrap_or(1))
}
```

**Why this works:** `std::process::Command` on Windows calls `CreateProcessW` directly. Each `arg()` becomes a separately quoted element in the Win32 argument list using MSVC CRT escaping rules. Paths with spaces are correctly handled without needing manual quoting or shell invocation.

### Pattern 3: Windows Binary Mode stdin/stdout for xargs -0

```rust
// Source: Rust forum — _setmode ABI discussion (users.rust-lang.org/t/appropriate-abi-for-windows-setmode/73360)
// Windows stdin/stdout default to text mode — \n↔\r\n translation corrupts null bytes
#[cfg(target_os = "windows")]
fn set_binary_mode() {
    extern "C" {
        fn _setmode(fd: i32, flags: i32) -> i32;
    }
    const _O_BINARY: i32 = 0x8000;
    unsafe {
        _setmode(0, _O_BINARY); // stdin
        _setmode(1, _O_BINARY); // stdout (for find -print0)
    }
}
```

**When to call:** Call before any stdin reads in `gow-xargs` with `-0`. Call before any stdout writes in `gow-find` with `-print0`. Call as the first thing after `gow_core::init()` when either flag is active.

**Important:** This uses `extern "C"` (cdecl), NOT `extern "system"` (stdcall). `_setmode` is a CRT function, not a Win32 API.

### Pattern 4: crossterm raw mode + panic-safe RAII guard for less

```rust
// Source: crossterm docs (docs.rs/crossterm/0.29.0) + Rust forum panic discussion
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use std::io::{self, stdout};
use std::panic;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Always runs: normal exit, panic, or unwinding
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn run_pager() -> Result<()> {
    // Install panic hook FIRST so panics restore terminal before printing
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let _guard = TerminalGuard; // restored on drop regardless of how we exit

    event_loop()?;
    Ok(())
}
```

**Critical:** Both the RAII guard AND the panic hook are needed. The guard handles normal exit and `?`-propagated errors. The panic hook handles `panic!()` and stack overflow. The `crossterm` library does NOT auto-restore terminal state on panic [VERIFIED: GitHub issue #368].

### Pattern 5: Line-offset index for less navigation (no OOM)

```rust
// Source: [ASSUMED] — standard pager implementation technique
// Decision D-09: never load full file into memory

pub struct LineIndex {
    offsets: Vec<u64>,  // byte offset of start of each line
    reader: BufReader<File>,
    eof_reached: bool,
}

impl LineIndex {
    pub fn new(file: File) -> Self {
        Self {
            offsets: vec![0],  // line 0 starts at byte 0
            reader: BufReader::new(file),
            eof_reached: false,
        }
    }
    
    // Scan forward to ensure `line_num` is indexed
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
    
    // G-jump: scan to EOF to index all lines, return total count
    pub fn scan_to_end(&mut self) -> io::Result<usize> {
        while !self.eof_reached {
            self.ensure_indexed_to(self.offsets.len())?;
        }
        Ok(self.offsets.len() - 1)
    }
    
    // Seek file to read content from specific line
    pub fn seek_to_line(&mut self, line_num: usize) -> io::Result<()> {
        let offset = self.offsets[line_num];
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}
```

**Memory characteristic:** Only `Vec<u64>` line offsets are stored (8 bytes per line). A 10 million-line file = ~80 MB of index, which is acceptable. The actual line content is read on demand for the visible viewport only.

**G-jump cost:** `scan_to_end()` must read the entire file sequentially on first `G` call. This is O(file size) but does not load content into memory. For stdin input (not seekable), the entire content must be buffered — use `Vec<u8>` accumulator then build the index in-memory (stdin is bounded by available RAM, which matches GNU less behavior).

### Pattern 6: crossterm event loop (keyboard handling)

```rust
// Source: docs.rs/crossterm/0.29.0/crossterm/event/
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

fn event_loop(state: &mut PagerState) -> Result<()> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _) => break,
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
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

### Pattern 7: xargs stdin tokenization

```rust
// Source: docs.rs/std/io/trait.BufRead (read_until)
use std::io::{self, BufRead};

fn tokenize_stdin(stdin: impl BufRead, null_delimited: bool) -> Vec<String> {
    let mut tokens = Vec::new();
    let delimiter = if null_delimited { b'\0' } else { b'\n' };
    let mut buf = Vec::new();
    let mut reader = stdin;
    loop {
        buf.clear();
        match reader.read_until(delimiter, &mut buf) {
            Ok(0) => break,
            Ok(_) => {
                // Remove trailing delimiter
                if buf.last() == Some(&delimiter) { buf.pop(); }
                // For newline mode: also strip \r
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

### Anti-Patterns to Avoid

- **Loading full file into memory in less:** Never `std::fs::read_to_string()` or `read_to_end()` for paging large files. Use the line-offset index and `File::seek` approach.
- **Shell intermediary in find -exec:** Never use `Command::new("cmd").arg("/C").arg(cmd)`. This re-introduces quoting bugs and is 10-50x slower than CreateProcess.
- **Text-mode stdin for xargs -0:** Never read null bytes without calling `_setmode(0, _O_BINARY)` first on Windows. The CRT text-mode translation will silently corrupt the stream (converts 0x1A to EOF).
- **Not restoring terminal on panic in less:** crossterm DOES NOT auto-restore raw mode or alternate screen on panic. The terminal will be left in a broken state visible to the user. Always use both RAII guard + panic hook.
- **Using `entry.path()` in -name matching:** GNU `find -name` matches the filename component only (basename). Using the full path would cause `-name "*.rs"` to match `/home/foo/src/bar.rs` incorrectly if `src` or `foo` also matched the pattern.
- **Treating -name as case-insensitive on Windows:** D-01 locks `-name` to case-sensitive. Do not change this even though Windows NTFS is case-insensitive by default.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive directory traversal with symlink tracking | Custom `fs::read_dir` recursion | `walkdir` 2.5 | Handles cycles, permissions, platform differences; already in workspace |
| Glob pattern compilation | `regex`-based glob-to-regex converter | `globset` 0.4 | Handles `[...]` character classes, Windows separator semantics, case folding correctly |
| Terminal raw mode on Windows | Win32 `SetConsoleMode` direct calls | `crossterm` 0.29 | Wraps `ENABLE_LINE_INPUT`/`ENABLE_ECHO_INPUT` bit manipulation; handles both VT100 and legacy Console API |
| Keyboard event reading in raw mode | `ReadConsoleInput` Win32 loop | `crossterm::event::read()` | Handles keyboard enhancement flags, resize events, multi-byte sequences |
| Pattern search in less | Custom string search | `regex` 1.12 | Already in workspace; provides case-insensitive option, find_iter() for match positions |

**Key insight:** The hardest part of this phase is `less` lifecycle management (raw mode + panic safety), not the data structures. Invest time in the RAII guard pattern before implementing rendering.

---

## Windows-Specific NTFS Timestamp Notes

**For find -mtime/-ctime/-atime:**

| Metadata field | Windows API | Behavior |
|----------------|------------|----------|
| `Metadata::modified()` | `ftLastWriteTime` | Always reliable on NTFS; maps to GNU `-mtime` |
| `Metadata::created()` | `ftCreationTime` | Windows creation time, not Unix ctime (which is inode change time). Close enough for most scripts. Maps to GNU `-ctime`. |
| `Metadata::accessed()` | `ftLastAccessTime` | **May equal `modified()` time.** Windows 10 with system volume >128 GB disables last-access updates by default (`NtfsDisableLastAccessUpdate = 0x80000003`). Maps to GNU `-atime`. |

**Registry key:** `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\NtfsDisableLastAccessUpdate`
- Value `0x80000000` or `0x80000002`: atime updates ENABLED
- Value `0x80000001` or `0x80000003`: atime updates DISABLED (default on large volumes)

**Implication for `-atime`:** Implement the flag correctly using `Metadata::accessed()`. Document in help text that results may be unreliable if atime tracking is disabled (per D-04). Do NOT attempt to read the registry at runtime — that is out of scope.

**Day calculation for -mtime/-atime/-ctime:**
```
GNU find uses: floor((now - mtime) / 86400)
+N means: more than N days ago
-N means: less than N days ago
N means: exactly N days ago (within 24h window)
```

Use `SystemTime::duration_since(UNIX_EPOCH)` with `Duration::as_secs()` for the calculation. [ASSUMED — standard POSIX find -mtime semantics; verified against GNU find manual]

---

## Common Pitfalls

### Pitfall 1: crossterm Terminal Not Restored on Panic
**What goes wrong:** If `less` panics while in raw mode (or alternate screen), the terminal is left in a broken state — no echo, no cursor, etc. The user must close and reopen their terminal.
**Why it happens:** crossterm does not register an atexit handler or panic hook automatically. The RAII Drop only fires for normal exit and `?` error propagation. A `panic!()` unwinds but still calls Drop — however panics that call `process::exit()` directly skip Drop entirely.
**How to avoid:** Implement both: (1) RAII guard struct with Drop impl that calls `disable_raw_mode()` + `LeaveAlternateScreen`, AND (2) `std::panic::set_hook()` that calls the same cleanup before the default handler. [VERIFIED: GitHub crossterm issue #368 confirms raw mode not auto-restored on panic]
**Warning signs:** User reports "terminal is broken after less crashes" — fix by adding panic hook.

### Pitfall 2: Windows stdin Silently Corrupting Null Bytes
**What goes wrong:** `xargs -0` reads zero bytes as EOF or gets corrupted records because Windows CRT text mode translates `0x1A` (Ctrl+Z) to EOF and `0x0D 0x0A` to `0x0A`.
**Why it happens:** Windows stdin opens in text mode by default. The CRT performs newline translation and treats `0x1A` as EOF marker.
**How to avoid:** Call `_setmode(0, _O_BINARY)` (stdin) before the first read when `-0` is active. Similarly, `gow-find -print0` must call `_setmode(1, _O_BINARY)` (stdout) before writing null-delimited output.
**Warning signs:** Pipeline `find -print0 | xargs -0 echo` produces incorrect output or silently drops paths — check binary mode initialization.

### Pitfall 3: find -name Matching Full Path Instead of Basename
**What goes wrong:** `-name "*.rs"` matches `/home/foo/src/bar.rs` because the path contains `src` which also contains `s`, or unexpectedly matches paths where a directory component matches the pattern.
**Why it happens:** Using `entry.path()` (full path) with the glob matcher instead of `entry.file_name()` (basename only).
**How to avoid:** Always call `entry.file_name()` and match against the basename. Only `-path` (not implemented in Phase 05) should match against the full path.
**Warning signs:** Integration test `find . -name "*.txt"` finds unexpected results; pattern `*.txt` matches directories named `foo.txt/`.

### Pitfall 4: WalkDir Filter Descending Into Excluded Directories
**What goes wrong:** `-maxdepth 1` still descends into subdirectories; or `-type f` excludes directory entries but still traverses them (wasting I/O).
**Why it happens:** Using `Iterator::filter()` on WalkDir entries doesn't prevent descent. `filter()` skips entries but still iterates them all.
**How to avoid:** Use `WalkDir::max_depth(n)` for depth control (built-in, no filter needed). For expensive predicate chains, note that `filter_entry()` prevents descent into directories that fail the predicate — but `-type` predicates should NOT be used with `filter_entry` (they should filter the yielded entries, not control descent).
**Warning signs:** `find . -maxdepth 0` lists files in subdirectories; very slow traversal on deep trees.

### Pitfall 5: less G-Jump Blocking the Event Loop
**What goes wrong:** Pressing `G` on a large file (e.g., 500 MB log) blocks the event loop for seconds while scanning to end.
**Why it happens:** `scan_to_end()` reads the entire file sequentially on first `G` call.
**How to avoid:** For the Phase 05 scope, a blocking scan on `G` is acceptable (GNU `less` also blocks on `G` for unknown-length streams). Document this behavior. For files opened with `File::seek`, the scan can be optimized later using `File::seek(SeekFrom::End(0))` to determine file size, then scanning backward — but this is a gap-closure optimization.
**Warning signs:** Less becomes unresponsive on `G` for files > 100 MB — note this in Phase 05 as a known limitation.

### Pitfall 6: crossterm EnterAlternateScreen Windows Issue
**What goes wrong:** On some Windows 11 + Windows Terminal configurations, `EnterAlternateScreen` may not work as expected (GitHub issue #973, reported 2025-02-22, currently unresolved).
**Why it happens:** Windows Terminal version-specific handling of the alternate screen ANSI escape sequence.
**How to avoid:** This is a platform bug outside our control. In integration tests, skip the alternate screen test on CI and add a note. Ensure `LeaveAlternateScreen` is always called in cleanup so the failure mode is non-destructive.
**Warning signs:** less displays content directly on the main terminal buffer rather than alternate screen on Windows.

---

## Code Examples

### globset case_insensitive API
```rust
// Source: docs.rs/globset/0.4.18/struct.GlobBuilder.html [VERIFIED]
use globset::{GlobBuilder, GlobMatcher};

// -name (case-sensitive, GNU default)
let matcher: GlobMatcher = GlobBuilder::new("*.rs")
    .case_insensitive(false)  // default
    .build()?.compile_matcher();

// -iname (case-insensitive)
let matcher: GlobMatcher = GlobBuilder::new("*.rs")
    .case_insensitive(true)
    .build()?.compile_matcher();

// Match only against file name component (not full path)
matcher.is_match(entry.file_name())
```

### crossterm keyboard event variants
```rust
// Source: docs.rs/crossterm/0.29.0/event/enum.KeyCode.html [VERIFIED]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Available KeyCode variants relevant to less:
// KeyCode::Up, Down, Left, Right
// KeyCode::PageUp, PageDown
// KeyCode::Home, End
// KeyCode::Char('q'), Char('/'), Char('n'), Char('N')
// KeyCode::Char('g'), Char('G'), Char('b'), Char(' ')
// KeyCode::Esc
// KeyCode::Enter
```

### WalkDir min_depth/max_depth API
```rust
// Source: docs.rs/walkdir/2.5.0/struct.WalkDir.html [VERIFIED]
use walkdir::WalkDir;

// Implements -maxdepth 2 -mindepth 1
let walker = WalkDir::new(root)
    .min_depth(min_depth)  // 0 = include root
    .max_depth(max_depth)  // usize::MAX = unlimited
    .follow_links(false);  // GNU find default: don't follow symlinks
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `windows` crate for Console API | `windows-sys` 0.61 (raw-dylib) | 0.61 release (2024) | No import lib download; used by all existing crates in workspace |
| Manual glob-to-regex | `globset` compiled matchers | Established | O(patterns) matching per file; correct `[...]` class handling |
| termion for terminal control | `crossterm` 0.29 (cross-platform) | N/A — termion was never cross-platform | Required for Windows |
| `std::io::stdin().lock()` binary reads on Windows | `_setmode(0, _O_BINARY)` before reads | Long-standing Windows CRT behavior | Prevents null byte corruption in xargs -0 pipeline |

**Deprecated/outdated:**
- `winapi` crate: unmaintained since 2020; superseded by `windows-sys`; do not add as dep
- `termion`: Unix-only; never appropriate for this project

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GNU `find -mtime` uses `floor((now - mtime) / 86400)` semantics with +N/N/-N modifiers | Windows-Specific NTFS Timestamp Notes | Wrong day calculation would fail compatibility tests against GNU find; low risk, easily testable |
| A2 | `less` G-jump blocking behavior for first invocation on large files is acceptable in Phase 05 scope | Pattern 5 (line-offset index) | User experience issue; acceptable if documented; gap-closure candidate |
| A3 | For stdin input to `less` (non-seekable), buffering all content in memory mirrors GNU less behavior | Pattern 5 | Could OOM on very large stdin streams; acceptable tradeoff for Phase 05 |
| A4 | `xargs -I {}` replacement should only replace literal `{}` tokens in the command arguments, not substrings | Standard Stack + xargs scope | Different from GNU xargs which replaces all occurrences of the replace-string; worth verifying against GNU xargs manual |

**If this table is empty:** All other claims in this research were verified or cited.

---

## Open Questions

1. **xargs -I {} substring replacement behavior**
   - What we know: D-12 locks the replacement string to `{}` only (not configurable)
   - What's unclear: Does `-I {}` replace only exact `{}` tokens, or all occurrences of `{}` within a larger argument? (e.g., `xargs -I {} echo "prefix/{}"`)
   - Recommendation: GNU xargs replaces all occurrences of the replace-string within each argument string. Implement substring replacement (`arg.replace("{}", &token)`) for correct GNU compatibility.

2. **crossterm EnterAlternateScreen on Windows Terminal (issue #973)**
   - What we know: Open issue from Feb 2025; no confirmed fix
   - What's unclear: Whether the issue affects all Windows Terminal versions or a specific release
   - Recommendation: Implement less to work without alternate screen as a graceful fallback; alternate screen is a UX improvement, not a hard requirement.

3. **find -print0 stdout binary mode timing**
   - What we know: `_setmode(1, _O_BINARY)` must be called before first stdout write
   - What's unclear: Whether `gow_core::init()` (which calls `SetConsoleOutputCP(65001)`) interferes with binary mode
   - Recommendation: Call `_setmode` immediately after `gow_core::init()` when `-print0` is active. Test `find -print0 | xargs -0 echo` as an integration test.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable toolchain | All three crates | ✓ | 1.95.0 (2026-04-14) | — |
| cargo | Build system | ✓ | 1.95.0 | — |
| `crossterm` 0.29 | gow-less | ✗ (not yet in workspace) | 0.29.0 on crates.io | — (must add to workspace Cargo.toml) |
| `globset` 0.4 | gow-find | ✗ (not yet in workspace) | 0.4.18 on crates.io | — (must add to workspace Cargo.toml) |
| `walkdir` 2.5 | gow-find | ✓ | 2.5.0 in workspace | — |
| `regex` 1.12 | gow-less (search) | ✓ | 1.12.3 in workspace | — |
| `terminal_size` 0.4 | gow-less (viewport) | ✓ | 0.4.0 in workspace | — |
| `assert_cmd` / `predicates` / `tempfile` | Integration tests | ✓ | All in workspace | — |
| Windows 10+ | crossterm alternate screen | ✓ | Windows 11 (target platform) | Graceful degradation per issue #973 |

**Missing dependencies with no fallback:**
- `crossterm = "0.29"` — must be added to `[workspace.dependencies]` in root `Cargo.toml`
- `globset = "0.4"` — must be added to `[workspace.dependencies]` in root `Cargo.toml`

**Missing dependencies with fallback:**
- None (all other deps are present)

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + assert_cmd 2.2.1 + predicates 3.1.4 |
| Config file | None — workspace uses `cargo test` directly |
| Quick run command | `cargo test --package gow-find --package gow-xargs --package gow-less` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| R015 | `find . -name "*.txt"` finds matching files | integration | `cargo test --package gow-find --test integration` | ❌ Wave 0 |
| R015 | `find . -name` is case-sensitive (D-01) | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -iname` is case-insensitive (D-01) | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -type f/d/l` filters by type | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -maxdepth N` limits depth | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -size +N` filters by size | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -mtime N` filters by mtime | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -exec echo {} \;` runs command per match | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -exec` handles paths with spaces (GOW #209) | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R015 | `find . -print0` writes null-delimited output | integration | `cargo test --package gow-find` | ❌ Wave 0 |
| R016 | `xargs echo` reads lines from stdin, runs echo | integration | `cargo test --package gow-xargs --test integration` | ❌ Wave 0 |
| R016 | `xargs -0 echo` reads null-delimited stdin | integration | `cargo test --package gow-xargs` | ❌ Wave 0 |
| R016 | `xargs -n 2 echo` batches max 2 args per invocation | integration | `cargo test --package gow-xargs` | ❌ Wave 0 |
| R016 | `xargs -I {} echo prefix/{}` does replacement | integration | `cargo test --package gow-xargs` | ❌ Wave 0 |
| R016 | `find -print0 \| xargs -0 echo` pipeline end-to-end | integration | `cargo test --package gow-xargs` | ❌ Wave 0 |
| R017 | `less file` displays content without OOM (non-interactive mode exits on EOF) | integration | `cargo test --package gow-less --test integration` | ❌ Wave 0 |
| R017 | `less` exits with code 0 on `q` (headless: auto-exit when not TTY) | integration | `cargo test --package gow-less` | ❌ Wave 0 |
| R017 | `less` handles ANSI passthrough | unit | `cargo test --package gow-less` | ❌ Wave 0 |
| R017 | `less` line index builds correctly for multi-line file | unit | `cargo test --package gow-less` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --package gow-{find,xargs,less}` for the crate being edited
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full workspace test suite green before `/gsd-verify-work`

### Integration Testing for less (headless strategy)

`less` is an interactive pager — it cannot be tested end-to-end in a real terminal in CI. The established strategy:

1. **Non-interactive fallback mode:** When stdout is not a TTY (`crossterm::tty::IsTty::is_tty()` returns false), `less` should behave like `cat` — print file contents to stdout and exit. This enables `assert_cmd` integration tests.

2. **Unit tests for core logic:** Test `LineIndex` (line indexing, seek), search matching, and ANSI detection without crossterm involvement. These run headlessly.

3. **No tests for raw mode keyboard interaction in CI:** The interactive event loop cannot be tested via assert_cmd. Document this as "manual UAT" in the verification plan.

```rust
// Non-interactive mode detection (headless CI compatibility)
use crossterm::tty::IsTty;
use std::io::stdout;

if !stdout().is_tty() {
    // Not a terminal — pipe mode: print content and exit
    return print_to_stdout(file_or_stdin);
}
// Terminal: enter interactive pager mode
```

### Wave 0 Gaps
- [ ] `crates/gow-find/tests/integration.rs` — covers R015 (all find predicates)
- [ ] `crates/gow-xargs/tests/integration.rs` — covers R016 (-0, -n, -I, pipeline)
- [ ] `crates/gow-less/tests/integration.rs` — covers R017 (headless non-TTY mode)
- [ ] `crates/gow-less/src/line_index.rs` unit tests — LineIndex struct coverage
- [ ] `crossterm = "0.29"` and `globset = "0.4"` added to `[workspace.dependencies]` in root `Cargo.toml`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | partial | `find -exec` inherits process permissions; no privilege escalation |
| V5 Input Validation | yes | Glob patterns from user input — `globset` handles invalid patterns via `build()` returning `Result`; exec command from user input passed directly to CreateProcess (correct behavior) |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| find -exec arbitrary command execution | Elevation of Privilege | `find -exec` is the intended behavior; no mitigation needed — user controls both find and exec |
| Malformed glob pattern causing error/panic | Denial of Service | `GlobBuilder::build()` returns `Err` on invalid patterns; handle with user-friendly error message, exit code 2 |
| Symlink traversal loops in find | Denial of Service | `walkdir` detects cycles when `follow_links(true)`; default is `follow_links(false)` which avoids loops |
| Null byte injection in xargs arg construction | Tampering | Rust `String` and `OsString` can contain null bytes; `Command::arg()` passes them through correctly to CreateProcess; not a security concern for this use case |

---

## Sources

### Primary (HIGH confidence)
- `C:\Users\super\workspace\gow-rust\Cargo.toml` — workspace deps verified (walkdir, regex, terminal_size, bstr present; crossterm, globset absent)
- `C:\Users\super\workspace\gow-rust\crates\gow-grep\src\lib.rs` — verified walkdir + WalkDir pattern in production use
- docs.rs/crossterm/0.29.0/crossterm/terminal/ — enable_raw_mode, disable_raw_mode, EnterAlternateScreen API
- docs.rs/crossterm/0.29.0/crossterm/event/enum.KeyCode.html — KeyCode variants (Up/Down/PageUp/PageDown/Char/Esc confirmed)
- docs.rs/globset/0.4.18/struct.GlobBuilder.html — case_insensitive() method signature confirmed
- docs.rs/walkdir/2.5.0/ — min_depth, max_depth, filter_entry, follow_links API confirmed
- crates.io crossterm 0.29.0 (verified 2026-04-28)
- crates.io globset 0.4.18 (verified 2026-04-28)

### Secondary (MEDIUM confidence)
- GitHub crossterm issue #368 — raw mode not restored on panic (confirmed as known behavior, no auto-fix in library)
- GitHub crossterm issue #973 — EnterAlternateScreen Windows 11 issue (open, unresolved as of Feb 2025)
- Rust forum users.rust-lang.org/t/appropriate-abi-for-windows-setmode/73360 — `extern "C"` for `_setmode`, `_O_BINARY = 0x8000` (community-verified)
- Microsoft Learn fsutil behavior — NtfsDisableLastAccessUpdate registry values and semantics

### Tertiary (LOW confidence)
- Line-offset index buffering strategy for less — standard pager technique, not verified against a specific codebase in this session [ASSUMED]
- `-mtime` day calculation semantics (floor division) [ASSUMED — standard POSIX semantics]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — workspace Cargo.toml verified; crates.io versions verified 2026-04-28
- Architecture: HIGH — patterns derived from production code in gow-grep/gow-tail; crossterm APIs confirmed
- Pitfalls: HIGH for raw mode/binary mode (GitHub issues + forum sources); MEDIUM for find semantics (GNU find manual not directly fetched)
- Environment: HIGH — `cargo --version` and `rustc --version` verified live

**Research date:** 2026-04-28
**Valid until:** 2026-05-28 (stable crates; crossterm/globset APIs unlikely to change)
