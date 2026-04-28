---
phase: "05-search-and-navigation"
plan: "04"
subsystem: "gow-less"
tags: [pager, crossterm, line-index, streaming-io, ansi-passthrough, raw-mode, raii, panic-hook]
dependency_graph:
  requires: ["05-01"]
  provides: ["gow-less binary", "LineIndex API", "non-TTY fallback for CI"]
  affects: ["gow-less/src/lib.rs", "gow-less/src/line_index.rs", "gow-less/tests/less_tests.rs"]
tech_stack:
  added: ["tempfile in [dependencies] (was only in [dev-dependencies])"]
  patterns:
    - "LineIndex: Vec<u64> byte-offset table, lazy forward scan via read_until+seek (D-09)"
    - "TerminalGuard RAII struct: Drop restores LeaveAlternateScreen + disable_raw_mode"
    - "panic::set_hook installed BEFORE enable_raw_mode (T-05-less-01 mitigation)"
    - "Non-TTY fallback: stdout.is_tty() == false -> io::copy cat mode (RESEARCH.md headless strategy)"
    - "ANSI passthrough: io::copy / write_all raw bytes without stripping ESC sequences (D-08)"
key_files:
  created:
    - "crates/gow-less/src/line_index.rs"
    - "crates/gow-less/tests/less_tests.rs"
  modified:
    - "crates/gow-less/src/lib.rs"
    - "crates/gow-less/Cargo.toml"
decisions:
  - "LineIndex uses ensure_indexed_to + scan_to_end pattern — forward scan only, seek for random access (D-09)"
  - "Stdin buffered to NamedTempFile for seekability — matches GNU less behavior (RESEARCH.md A3)"
  - "G-jump blocks on first call to scan_to_end (Pitfall 5 / A2) — documented known limitation"
  - "ANSI passthrough via raw io::copy in non-TTY mode and write_all(line_bytes) in TTY render"
  - "tempfile added to [dependencies] because open_source() uses NamedTempFile at runtime (not just in tests)"
metrics:
  duration: "5m 26s"
  completed: "2026-04-28T02:15:40Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 2
  files_modified: 2
---

# Phase 05 Plan 04: gow-less Pager Implementation Summary

Implemented `gow-less` — a real interactive terminal pager with lazy line indexing, raw mode lifecycle management, ANSI passthrough, and full headless CI coverage via non-TTY fallback.

## What Was Built

### Task 1: LineIndex (lazy line-offset index)
`crates/gow-less/src/line_index.rs` — 262 lines

The core data structure for D-09 (no full-file load). Stores `Vec<u64>` of line-start byte offsets — only 8 bytes per line regardless of line length. Line content is read on demand via `File::seek(SeekFrom::Start(offset))` + `read_until(b'\n')`.

Key API:
- `LineIndex::new(file: File)` — initializes with `offsets = [0]`, `eof_reached = false`
- `ensure_indexed_to(line_num)` — reads forward until `offsets.len() > line_num` or EOF
- `scan_to_end()` — reads to EOF, returns total line count (used by `G` jump)
- `read_line_at(line_num)` — seeks to stored offset, reads one line; returns `Ok(None)` past EOF
- `line_count_so_far()` — `offsets.len() - 1` (trailing EOF marker not counted)
- `is_complete()` — true once EOF reached

Unicode correctness: offsets are byte-counted via `n` return value from `read_until`, not character-counted. `test_unicode_byte_offsets_correct` verifies Korean (3-byte UTF-8 chars) offsets exactly match.

6 unit tests: empty file, single line without newline, 3-line, lazy (D-09 proof), seek-back after scan, Unicode byte offsets.

### Task 2: Full lib.rs Implementation
`crates/gow-less/src/lib.rs` — 398 lines

**Non-TTY fallback (RESEARCH.md headless strategy):**
```rust
if !io::stdout().is_tty() {
    return match copy_to_stdout(&cli) { ... };
}
```
When stdout is piped (e.g. under `assert_cmd`, `| head`), the binary behaves like `cat` — `io::copy` passes bytes byte-for-byte including ANSI escape sequences (D-08).

**Stdin handling:** Non-seekable stdin is buffered to a `tempfile::NamedTempFile` before building `LineIndex`. This mirrors GNU `less` behavior (RESEARCH.md A3).

**Panic-safe terminal lifecycle (T-05-less-01 mitigation):**
```rust
// 1. Install panic hook FIRST — before enable_raw_mode
let default_hook = panic::take_hook();
panic::set_hook(Box::new(move |info| {
    let _ = io::stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
    default_hook(info);
}));

// 2. THEN enter raw mode
enable_raw_mode()?;
io::stdout().execute(EnterAlternateScreen)?;

// 3. RAII guard handles every other exit path (?-propagation, normal return)
let _guard = TerminalGuard;
```
Both mechanisms are required: the hook handles `panic!()` and stack overflow; the guard handles graceful exit.

**PagerState:** `top_line`, `viewport_h/w`, `search_pattern: Option<Regex>`, `match_lines: Vec<usize>`, `current_match_idx`. Render writes raw bytes via `write_all` (D-08 ANSI passthrough in TTY mode too).

**Event loop implements D-07:** Arrow keys + j/k scroll, PgUp/PgDn/b/space jump by page, g/G, `/`-search with n/N, q/Ctrl-C exit, Resize event.

**Search:** Collects keystrokes until Enter, compiles to `Regex` (linear-time, no catastrophic backtracking — T-05-less-04), scans indexed lines plus extends forward up to 100K lines, stores match_lines for n/N navigation.

**Cargo.toml change:** `tempfile` moved from `[dev-dependencies]` to `[dependencies]` because `open_source()` uses `NamedTempFile` at runtime (not just in tests).

### Task 3: Headless Integration Tests
`crates/gow-less/tests/less_tests.rs` — 175 lines, 7 tests

| Test | Coverage |
|------|----------|
| `test_less_file_arg_prints_full_contents_in_non_tty_mode` | D-07: file mode, exit 0, all lines present |
| `test_less_stdin_pass_through_in_non_tty_mode` | D-07: stdin pipe pass-through |
| `test_less_preserves_ansi_escape_bytes` | D-08: byte-exact `\x1b[31m...\x1b[0m\n` assertion |
| `test_less_handles_unicode_content` | D-07 + UTF-8: Korean multi-byte chars |
| `test_less_large_file_streams_without_oom` | D-09: 1 MiB / 10K lines, first+last line in stdout |
| `test_less_missing_file_errors_with_exit_1` | Error handling: exit 1, `less: ` prefix in stderr |
| `test_less_empty_file_succeeds` | Edge case: empty file exits 0 with empty stdout |

## Known Limitations

- **G (jump to end) blocks event loop** on large files during first invocation — `scan_to_end()` reads the entire file sequentially (Pitfall 5 / A2). For typical log files this is fast enough. Gap-closure candidate: use `File::seek(SeekFrom::End(0))` + backward scan.
- **Interactive raw mode not tested in CI** — impossible via `assert_cmd` (no real TTY). Interactive keys, alternate screen, and panic recovery verified via manual UAT only (see 05-VALIDATION.md).
- **Stdin buffered to disk** — large piped stdin writes to a temp file (bounded by disk, not RAM). Matches GNU `less` behavior.
- **Search capped at 100K lines** — `find_all_matches` stops scanning forward after 100 000 lines. Safety cap; removable in gap-closure plan.

## Test Results

```
cargo test -p gow-less
  Unit tests (lib):   7 passed (6 LineIndex + 1 symbol check)
  Integration tests:  7 passed
  Total:             14 passed, 0 failed
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy warnings treated as errors**
- **Found during:** Task 2 (`cargo clippy -p gow-less -- -D warnings`)
- **Issue 1:** `self.offsets.len() == 0` in `line_index.rs` → clippy `len_zero`
- **Issue 2:** `match event::read()?` with single arm in `enter_search` → clippy `single_match`
- **Fix:** Changed to `self.offsets.is_empty()` and `if let Event::Key(...)` pattern
- **Files modified:** `crates/gow-less/src/line_index.rs`, `crates/gow-less/src/lib.rs`
- **Commit:** included in Task 2 commit (`174eafd`)

**2. [Rule 1 - Bug] Unused import `Read` in lib.rs**
- **Found during:** Task 2 initial build
- **Issue:** `use std::io::{self, Read, Write}` — `Read` not actually used (io::copy handles it internally)
- **Fix:** Removed `Read` from the import
- **Files modified:** `crates/gow-less/src/lib.rs`
- **Commit:** included in Task 2 commit (`174eafd`)

## Threat Coverage

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-05-less-01: Terminal corruption on panic | `panic::set_hook` BEFORE `enable_raw_mode` + `TerminalGuard RAII` | Source-order assertion in acceptance criteria |
| T-05-less-02: Memory OOM on large files | `LineIndex` only stores `Vec<u64>` offsets (~8 bytes/line) | `test_less_large_file_streams_without_oom` |
| T-05-less-04: Pathological regex | `regex` crate (linear-time guaranteed) | Per CLAUDE.md stack mandate |
| T-05-less-07: Regex compile failure | `Regex::new` returns `Result`; pager continues on Err | Search `enter_search` code path |
| T-05-less-08: Stub binary in place | Stub text `less: not implemented` absent from lib.rs | Task 2 acceptance criteria |

## Self-Check: PASSED

Verified:
- `crates/gow-less/src/line_index.rs` exists: FOUND
- `crates/gow-less/src/lib.rs` contains `pub fn uumain`: FOUND
- `crates/gow-less/tests/less_tests.rs` exists: FOUND
- Task 1 commit e2a4dce: FOUND
- Task 2 commit 174eafd: FOUND
- Task 3 commit 856a418: FOUND
- `cargo test -p gow-less`: 14 passed, 0 failed
