---
phase: 05-search-and-navigation
verified: 2026-04-28T04:00:00Z
status: human_needed
score: 21/25 must-haves verified (4 require human/TTY testing)
overrides_applied: 0
human_verification:
  - test: "Launch 'cargo run -p gow-less -- <any large file>' in a real terminal. Press arrow keys, j/k, PgUp/PgDn, b/Space. Verify scrolling works correctly."
    expected: "Viewport scrolls up and down; content updates on each keypress; no garbled output"
    why_human: "crossterm raw mode requires a real TTY — assert_cmd pipes stdout so the interactive path is never entered in CI"
  - test: "In the same session, type '/foo' (for some pattern that exists), press Enter. Then press n and N repeatedly."
    expected: "Viewport jumps to first match; n/N cycle through matches in forward and reverse order"
    why_human: "Interactive search mode requires reading keystrokes in raw mode — untestable headlessly"
  - test: "Press g to jump to the top, then G to jump to the bottom of the file."
    expected: "g sets viewport to line 0; G scans to EOF and positions at last line. Terminal remains responsive (no freeze on small files)"
    why_human: "Event loop key dispatch requires a real terminal"
  - test: "Press q. Verify the terminal is fully restored: prompt is on a clean line, echo is active, no stray artifacts."
    expected: "Shell prompt returns normally with no broken cursor or raw-mode residue"
    why_human: "RAII TerminalGuard + LeaveAlternateScreen can only be verified by inspecting the actual terminal state"
---

# Phase 05: search-and-navigation Verification Report

**Phase Goal:** Implement find, xargs, and less utilities — GNU-compatible file search, argument batching, and interactive paging for Windows.
**Verified:** 2026-04-28T04:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

All truths sourced from the merged set of ROADMAP success criteria (5) and PLAN must_haves across 05-01 through 05-04.

#### ROADMAP Success Criteria

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | `find` traverses directory trees with `-name`, `-type`, `-size`, `-mtime` predicates and executes commands via `-exec cmd {} \;` | VERIFIED | `crates/gow-find/src/lib.rs` (603 lines): `GlobBuilder::new`, `WalkDir::new`, `follow_links(false)`, `parse_size_spec`, `parse_time_spec`, `exec_for_entry`. All 13 integration tests pass including `test_exec_runs_command_per_match`, `test_exec_handles_paths_with_spaces`, `test_mtime_recent_files`, `test_size_greater_than` |
| SC-2 | `xargs` reads stdin and builds command lines with `-0`, `-I {}`, `-n`, `-L` flags | VERIFIED | `crates/gow-xargs/src/lib.rs` (329 lines): `tokenize_stdin`, `exec_batch`, `exec_with_replacement`, `aggregate_exit`. All 8 integration tests pass including `test_xargs_null_mode_reads_nul_separated`, `test_xargs_replace_braces_substring`, `test_xargs_n_batches_args`, `test_xargs_L_batches_lines` |
| SC-3 | `less` pages files interactively with scroll, `/` search, and ANSI color passthrough | PARTIAL — automated portion verified; interactive TTY behavior requires human | Non-TTY path fully verified: 7/7 integration tests pass, ANSI byte-equality test confirms D-08. Interactive event loop is structurally complete (scroll, search, g/G, q/Ctrl-C all wired) but requires human UAT |
| SC-4 | `find -print0 \| xargs -0 cmd` pipeline works end-to-end on Windows | VERIFIED | `test_pipeline_find_print0_into_xargs_0` in `xargs_tests.rs` passes — uses `Stdio::piped()`, no shell, proves NUL bytes survive Windows pipe. Both `alpha.txt` and `beta.txt` appear in xargs output |
| SC-5 | All three binaries compile cleanly as independent crates in the workspace | VERIFIED | `cargo build --workspace` exits 0. All three are members of the workspace Cargo.toml (`crates/gow-find`, `crates/gow-xargs`, `crates/gow-less`). `crossterm = "0.29"` and `globset = "0.4"` declared in `[workspace.dependencies]` |

#### Plan Must-Have Truths (05-01)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| P01-1 | Root Cargo.toml lists crates/gow-find, crates/gow-xargs, crates/gow-less as workspace members | VERIFIED | `grep "crates/gow-find\|crates/gow-xargs\|crates/gow-less" Cargo.toml` — all three present |
| P01-2 | Root Cargo.toml [workspace.dependencies] declares crossterm = "0.29" and globset = "0.4" | VERIFIED | Both found in Cargo.toml workspace.dependencies section |
| P01-3 | `cargo build --workspace` exits 0 with all three new stub crates compiling | VERIFIED | Build succeeds; only pre-existing gow-awk warnings (unrelated to phase 05) |
| P01-4 | Each new crate builds the binary name expected by GNU users (find, xargs, less) | VERIFIED | `name = "find"`, `name = "xargs"`, `name = "less"` confirmed in respective Cargo.toml [[bin]] sections |

#### Plan Must-Have Truths (05-02 — find)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| P02-1 | `find <dir> -name '*.txt'` lists every regular file whose basename matches the glob, recursively (case-sensitive) | VERIFIED | `test_name_matches_basename_only` and `test_name_is_case_sensitive` pass |
| P02-2 | `find <dir> -iname '*.TXT'` performs the same match case-insensitively | VERIFIED | `test_iname_is_case_insensitive` passes; `GlobBuilder::new().case_insensitive(true)` confirmed in lib.rs |
| P02-3 | `-type f\|d\|l` filters output to files / directories / symlinks | VERIFIED | `test_type_filter_files` and `test_type_filter_directories` pass |
| P02-4 | `-size +Nk / -Nk / Nk` filters by file size with k/M/G unit suffixes | VERIFIED | `test_size_greater_than` passes; `parse_size_spec` unit tests cover `+10k`, `-1M`, `5`, `100c`, `+2G` |
| P02-5 | `-mtime / -atime / -ctime` with `+N / -N / N` filters by time in days | VERIFIED | `test_mtime_recent_files` passes; `parse_time_spec` unit tests green |
| P02-6 | `-maxdepth N / -mindepth N` controls traversal depth | VERIFIED | `test_maxdepth_zero_lists_root_only` and `test_maxdepth_one_skips_subdir_contents` pass |
| P02-7 | `-exec cmd {} \;` runs cmd once per match via std::process::Command — handles paths with spaces | VERIFIED | `test_exec_runs_command_per_match` and `test_exec_handles_paths_with_spaces` pass; `exec_for_entry` uses `Command::new(cmd).args(&args)` — no shell, each arg is separate |
| P02-8 | `-print0` writes paths separated by NUL bytes with stdout in binary mode | VERIFIED | `test_print0_emits_null_separated_paths` asserts `out.stdout.contains(&0u8)` and `nul_count >= 2`; `_setmode(1, _O_BINARY)` confirmed in lib.rs |
| P02-9 | Multiple predicates AND together | VERIFIED | `test_multiple_predicates_and_together` passes |
| P02-10 | Invalid glob patterns produce stderr message and exit code 2 | VERIFIED | `build_name_matcher` returns `Err(anyhow!("invalid glob '{}': ..."))` — `uumain` maps to exit 2 |

#### Plan Must-Have Truths (05-03 — xargs)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| P03-1 | `xargs cmd` reads stdin lines, passes each as an argument to cmd | VERIFIED | `test_xargs_default_newline_mode_appends_args` passes |
| P03-2 | `xargs -0 cmd` reads NUL-separated stdin (binary mode on Windows) | VERIFIED | `test_xargs_null_mode_reads_nul_separated` passes; `_setmode(0, _O_BINARY)` in lib.rs |
| P03-3 | `xargs -n N cmd` batches at most N arguments per invocation | VERIFIED | `test_xargs_n_batches_args` passes — asserts 2 output lines for 4 tokens with `-n 2` |
| P03-4 | `xargs -L N cmd` batches at most N input lines per invocation | VERIFIED | `test_xargs_L_batches_lines` passes |
| P03-5 | `xargs -I {} cmd {}` performs substring replacement of `{}`, runs once per token | VERIFIED | `test_xargs_replace_braces_substring` passes; `replace_braces` function confirmed |
| P03-6 | Spawned commands inherit caller stdout/stderr; xargs exits 0/123/124/125 per GNU | VERIFIED | `aggregate_exit` function present; unit tests `test_aggregate_exit_all_success` (0), `test_aggregate_exit_some_failure` (123), `test_aggregate_exit_signal_dominates` (124) pass |
| P03-7 | Pipeline `find -print0 \| xargs -0 cmd` works end-to-end on Windows | VERIFIED | `test_pipeline_find_print0_into_xargs_0` passes — real Stdio::piped pipeline, no shell |

#### Plan Must-Have Truths (05-04 — less)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| P04-1 | `less <file>` in non-TTY mode prints file's full contents to stdout and exits 0 | VERIFIED | `test_less_file_arg_prints_full_contents_in_non_tty_mode` passes |
| P04-2 | `less` reading from stdin in non-TTY mode passes stdin through to stdout | VERIFIED | `test_less_stdin_pass_through_in_non_tty_mode` passes |
| P04-3 | `less` in TTY mode enables raw mode + alternate screen, restores terminal on normal exit, on panic, and on Ctrl-C | HUMAN NEEDED | `TerminalGuard` RAII + `panic::set_hook` structurally verified (lines 143/149 confirm hook before raw mode). Cannot verify actual terminal restoration without a real TTY |
| P04-4 | Arrow keys, PgUp/PgDn, j/k scroll the viewport | HUMAN NEEDED | Event loop key bindings for scroll are wired in `event_loop` (lines 352-379); execution requires a real terminal |
| P04-5 | q and Ctrl-C exit the pager cleanly | HUMAN NEEDED | Wired in event loop; requires real TTY to verify terminal restoration |
| P04-6 | g jumps to first line, G scans to EOF and jumps to last line | HUMAN NEEDED | `jump_to_start()` and `jump_to_end()` wired; requires real TTY |
| P04-7 | `/pattern` enters search mode; n/N navigate next/previous match | HUMAN NEEDED | `enter_search()`, `next_match()`, `prev_match()` wired; requires real TTY |
| P04-8 | ANSI escape sequences pass through unchanged | VERIFIED | `test_less_preserves_ansi_escape_bytes` asserts byte-exact `\x1b[31mRED TEXT\x1b[0m\n` in stdout |
| P04-9 | LineIndex never reads the whole file at once — lazy Vec<u64> of byte offsets + File::seek | VERIFIED | `test_less_large_file_streams_without_oom` (1 MiB, 10K lines) passes; `test_lazy_indexing_does_not_scan_full_file` confirms `!is_complete()` after partial index |

**Score:** 21/25 truths automated-verified; 4 require human/TTY interaction (all in the interactive pager path — structurally implemented and wired, not stubs)

### Deferred Items

None. All phase 05 scope is present in the current codebase.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | Workspace member registration + crossterm/globset deps | VERIFIED | All three crates present in members array; both deps in [workspace.dependencies] |
| `crates/gow-find/src/lib.rs` | Real implementation: Cli struct, predicates, WalkDir, exec, print0 | VERIFIED | 603 lines; min_lines=250 exceeded; all key patterns present |
| `crates/gow-find/src/main.rs` | 3-line main.rs delegating to uu_find::uumain | VERIFIED | Line 2: `std::process::exit(uu_find::uumain(std::env::args_os()))` |
| `crates/gow-find/Cargo.toml` | [lib] name = uu_find, [[bin]] name = find | VERIFIED | Both confirmed |
| `crates/gow-find/build.rs` | Windows manifest embedding | VERIFIED | `embed_manifest::embed_manifest` + `ActiveCodePage::Utf8` present |
| `crates/gow-find/tests/find_tests.rs` | Integration tests for all find predicates | VERIFIED | 13 tests (exceeds min 12); all named tests present and passing |
| `crates/gow-xargs/src/lib.rs` | Real implementation: tokenizer, binary mode, exec batch/replace | VERIFIED | 329 lines; min_lines=200 exceeded; all key patterns present |
| `crates/gow-xargs/Cargo.toml` | [lib] name = uu_xargs, [[bin]] name = xargs | VERIFIED | Both confirmed; no regex or walkdir (correct) |
| `crates/gow-xargs/build.rs` | Windows manifest embedding | VERIFIED | Present |
| `crates/gow-xargs/tests/xargs_tests.rs` | Integration tests for xargs flags + pipeline | VERIFIED | 8 tests (exceeds min 7); pipeline test uses Stdio::piped() |
| `crates/gow-less/src/line_index.rs` | LineIndex struct with lazy offset indexing | VERIFIED | 262 lines; min_lines=100 exceeded; all methods present |
| `crates/gow-less/src/lib.rs` | Full pager: Cli, non-TTY fallback, RAII guard, panic hook, event loop | VERIFIED | 395 lines; min_lines=250 exceeded; all required symbols present |
| `crates/gow-less/Cargo.toml` | [lib] name = uu_less, [[bin]] name = less; tempfile in [dependencies] | VERIFIED | Both confirmed; tempfile in both [dependencies] (runtime) and [dev-dependencies] |
| `crates/gow-less/build.rs` | Windows manifest embedding | VERIFIED | Present |
| `crates/gow-less/tests/less_tests.rs` | Headless integration tests: non-TTY, ANSI, large file | VERIFIED | 7 tests; all named tests present and passing |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Cargo.toml` | `crates/gow-find/Cargo.toml` | workspace members array | WIRED | Pattern `"crates/gow-find"` confirmed |
| `Cargo.toml` | `crates/gow-xargs/Cargo.toml` | workspace members array | WIRED | Pattern `"crates/gow-xargs"` confirmed |
| `Cargo.toml` | `crates/gow-less/Cargo.toml` | workspace members array | WIRED | Pattern `"crates/gow-less"` confirmed |
| `crates/gow-find/Cargo.toml` | `Cargo.toml [workspace.dependencies]` | `globset = { workspace = true }` | WIRED | Confirmed in Cargo.toml |
| `crates/gow-less/Cargo.toml` | `Cargo.toml [workspace.dependencies]` | `crossterm = { workspace = true }` | WIRED | Confirmed in Cargo.toml |
| `crates/gow-find/src/lib.rs` | `globset::GlobBuilder` | `build_name_matcher()` | WIRED | `GlobBuilder::new` at line 334 |
| `crates/gow-find/src/lib.rs` | `walkdir::WalkDir` | `.min_depth()/.max_depth()/.follow_links(false)` | WIRED | `WalkDir::new` at line 185, `follow_links(false)` confirmed |
| `crates/gow-find/src/lib.rs` | `std::process::Command` | `exec_for_entry()` | WIRED | `Command::new(&cmd_parts[0]).args(&args)` at line 457 |
| `crates/gow-find/src/lib.rs` | `_setmode (windows CRT)` | `set_stdout_binary_mode()` when `-print0` is active | WIRED | `extern "C" fn _setmode` in `#[cfg(target_os = "windows")]` block |
| `crates/gow-xargs/src/lib.rs` | `_setmode (windows CRT)` | `set_stdin_binary_mode()` when `-0` is active | WIRED | `extern "C" fn _setmode` in `#[cfg(target_os = "windows")]` block; called at line 91 |
| `crates/gow-xargs/src/lib.rs` | `std::io::BufRead::read_until` | `tokenize_stdin()` loop | WIRED | `reader.read_until(delimiter, &mut buf)` at line 165 |
| `crates/gow-xargs/src/lib.rs` | `std::process::Command` | `exec_batch`/`exec_with_replacement` | WIRED | `Command::new(cmd).args(&all_args)` at lines 207/220 |
| `crates/gow-less/src/lib.rs` | `crates/gow-less/src/line_index.rs` | `pub mod line_index;` | WIRED | Line 10: `pub mod line_index;` |
| `crates/gow-less/src/lib.rs` | `crossterm::terminal::enable_raw_mode` | `run_pager()` | WIRED | Line 149: `enable_raw_mode()?` |
| `crates/gow-less/src/lib.rs` | `crossterm::terminal::disable_raw_mode` | `TerminalGuard::drop` and panic hook | WIRED | Lines 133, 146 |
| `crates/gow-less/src/lib.rs` | `std::panic::set_hook` | `run_pager()` — BEFORE `enable_raw_mode` | WIRED | Lines 143 (set_hook) vs 149 (enable_raw_mode) — correct source order confirmed |
| `crates/gow-less/src/lib.rs` | `crossterm::tty::IsTty` | `uumain` — non-TTY fallback branch | WIRED | Line 66: `if !io::stdout().is_tty()` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `gow-find/src/lib.rs` | `entry` (DirEntry) | `WalkDir::new(root).into_iter()` | Yes — walks real filesystem | FLOWING |
| `gow-xargs/src/lib.rs` | `tokens` | `tokenize_stdin(reader, cli.null)` | Yes — reads from real stdin via `read_until` | FLOWING |
| `gow-less/src/lib.rs` (non-TTY) | file bytes | `File::open(path)` + `io::copy` | Yes — reads real file | FLOWING |
| `gow-less/src/line_index.rs` | `offsets` | `BufReader<File>` + `read_until(b'\n')` | Yes — builds real byte offsets | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `cargo build --workspace` exits 0 | `cargo build --workspace` | Finished `dev` profile (0.23s) | PASS |
| `cargo test -p gow-find --lib` — 15 unit tests | `cargo test -p gow-find --lib` | 15 passed, 0 failed | PASS |
| `cargo test -p gow-find` — 13 integration tests | `cargo test -p gow-find` | 13 passed, 0 failed | PASS |
| `cargo test -p gow-xargs --lib` — 11 unit tests | `cargo test -p gow-xargs --lib` | 11 passed, 0 failed | PASS |
| `cargo test -p gow-xargs` — 8 integration tests + pipeline | `cargo test -p gow-xargs` | 8 passed, 0 failed (includes pipeline test) | PASS |
| `cargo test -p gow-less --lib` — 7 unit tests | `cargo test -p gow-less --lib` | 7 passed, 0 failed | PASS |
| `cargo test -p gow-less` — 7 integration tests | `cargo test -p gow-less` | 7 passed, 0 failed | PASS |
| `panic::set_hook` installed before `enable_raw_mode` | Source order check | Line 143 vs 149 | PASS |
| No stub text remaining | `grep "not implemented"` across 3 lib.rs files | No matches | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| R015 | 05-02 | 파일 검색 (-name, -type, -size, -mtime 등), -exec 지원, 공백 처리 | SATISFIED | Full `find` implementation with all FIND-01/02/03 predicates; GOW #208 (#exec) and #209 (spaces) regressed by integration tests |
| R016 | 05-03 | 표준 입력에서 명령줄 구성 (-0 null 구분, -I 치환) | SATISFIED | Full `xargs` implementation with XARGS-01 flag set; pipeline test confirms Windows binary-mode pipe safety |
| R017 | 05-04 | 파일 페이저 (스크롤, 검색, 큰 파일 지원) | SATISFIED (automated) / HUMAN NEEDED (interactive) | Non-TTY path fully tested (ANSI, large file, unicode); interactive TTY path structurally complete but requires human UAT per LESS-01 |

No orphaned requirements: REQUIREMENTS.md maps R015, R016, R017 to M001/S05 — all three are claimed by plans 05-02, 05-03, 05-04 respectively.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/gow-less/src/lib.rs` | 306 | `if line > 100_000 { break; }` — search cap | Info | Known and documented limitation: `/pattern` search stops scanning after 100K lines. Mentioned in SUMMARY as gap-closure candidate. Does NOT affect any test or current use case |
| `crates/gow-less/src/lib.rs` | 237 | `scan_to_end()` called synchronously in `G` key handler | Info | Documented known limitation (Pitfall 5 / A2): first `G` press blocks on large files. Acceptable for Phase 05 per RESEARCH.md; gap-closure candidate |

No blockers. No stubs. No hardcoded empty data flowing to rendered output.

### Human Verification Required

#### 1. Interactive Scroll and Navigation

**Test:** Open a real terminal. Run `cargo run -p gow-less -- <some-file-with-many-lines>` (e.g. the workspace `Cargo.lock`). Press: Arrow Down, Arrow Up, j, k, PgDn, PgUp, b, Space.
**Expected:** Viewport scrolls in all four directions. Content updates immediately on each keypress. No garbled characters.
**Why human:** crossterm raw mode requires a real TTY. `assert_cmd` pipes stdout, so the binary enters the non-TTY (cat) path — the interactive event loop never runs in CI.

#### 2. Search with n/N Navigation

**Test:** In the same interactive session, type `/` then a pattern that exists in the file (e.g. `/package`), then press Enter. Press `n` several times, then `N`.
**Expected:** Viewport jumps to the first match after `/`-Enter. Each `n` press advances to the next match; each `N` press steps back. Status line shows the active pattern.
**Why human:** `enter_search()` reads keystrokes via `event::read()` in raw mode — only possible with a real terminal.

#### 3. g / G Jump Keys

**Test:** In the same session, press `g`. Then press `G`.
**Expected:** `g` immediately returns to line 0 (top of file). `G` scans to EOF and positions the viewport at the last lines.
**Why human:** Key dispatch in `event_loop` requires real TTY.

#### 4. Clean Exit and Terminal Restoration

**Test:** After any interactive session, press `q` (or Ctrl-C). Inspect the terminal state.
**Expected:** Shell prompt appears on a clean line. Text input (echo) works normally. No stray `\x1b[?1049h` (alternate screen) residue. Cursor visible and positioned correctly.
**Why human:** `TerminalGuard::drop` calls `LeaveAlternateScreen` + `disable_raw_mode`. Verifying the actual terminal state requires eyes on the terminal — not programmable from outside.

### Gaps Summary

No blocking gaps. The phase goal "Implement find, xargs, and less utilities — GNU-compatible file search, argument batching, and interactive paging for Windows" is achieved:

- `find` is fully implemented with all locked predicates and `-exec`. All 13 integration tests pass.
- `xargs` is fully implemented with `-0`, `-I {}`, `-n`, `-L`. The cross-binary `find -print0 | xargs -0` pipeline test passes on Windows.
- `less` has a fully implemented interactive TTY pager (structurally complete and wired), plus a headless non-TTY path that is 100% tested (ANSI passthrough, large files, unicode, stdin, error handling). The 4 human-needed items are the TTY interactive behaviors — these are inherently untestable in CI and require manual UAT.

The `status: human_needed` reflects that SC-3 (interactive pager) and 4 of the 05-04 must-have truths cannot be verified without a real terminal session, per the phase plan's explicit strategy (see 05-VALIDATION.md "Manual-Only Verifications").

---

_Verified: 2026-04-28T04:00:00Z_
_Verifier: Claude (gsd-verifier)_
