---
phase: "05-search-and-navigation"
plan: "03"
subsystem: "search-and-navigation"
tags: [xargs, windows-binary-mode, nul-pipeline, argument-batching, replacement-mode, integration-tests]
dependency_graph:
  requires: [05-01]
  provides: [gow-xargs-implementation, xargs-integration-tests, find-print0-pipeline]
  affects:
    - crates/gow-xargs/src/lib.rs
    - crates/gow-xargs/tests/xargs_tests.rs
    - crates/gow-find/src/lib.rs
tech_stack:
  added: []
  patterns:
    - set_stdin_binary_mode via _setmode CRT (extern "C", NOT extern "system")
    - tokenize_stdin using BufRead::read_until for NUL and newline delimiter modes
    - aggregate_exit: 0/123/124/125 GNU-compatible xargs exit codes
    - exec_batch / exec_with_replacement returning Option<i32> for signal vs exit discrimination
    - replace_braces for literal {} substring replacement (-I {} semantics)
    - cross-binary pipeline test via Stdio::piped() (no shell intermediary)
key_files:
  created:
    - crates/gow-xargs/tests/xargs_tests.rs
  modified:
    - crates/gow-xargs/src/lib.rs
    - crates/gow-find/src/lib.rs
decisions:
  - "extern 'C' requires 'unsafe' in Rust 2024 edition — used 'unsafe extern \"C\"' for _setmode declaration"
  - "exec_batch/exec_with_replacement return Option<i32> (not i32) to distinguish signal-killed (None) from exit code"
  - "Minimal gow-find bootstrap implemented (Rule 3 deviation) — find stub blocked cross-binary pipeline test acceptance criteria"
  - "test_xargs_L_batches_lines and test_xargs_I_rejects_combined_n retain uppercase in names per plan; #[allow(non_snake_case)] added at module level"
  - "GNU 4.4+ default (no-run-if-empty): empty stdin returns 0 without running command"
metrics:
  duration_seconds: 330
  completed_date: "2026-04-28"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 2
---

# Phase 05 Plan 03: gow-xargs Implementation — Summary

**One-liner:** GNU xargs with -0 (NUL-safe Windows binary mode via _setmode), -I {} (substring replacement), -n/-L (batching), GNU exit codes (0/123/124/125), and a cross-binary `find -print0 | xargs -0` pipeline integration test.

## What Was Built

### Task 1: gow-xargs core implementation (`crates/gow-xargs/src/lib.rs`)

Replaced the stub with a full implementation of D-12's flag set:

**Cli struct flags:**
- `-0` / `--null`: NUL-separated input mode; calls `set_stdin_binary_mode()` before first read
- `-I` / `--replace`: fixed `{}` substring replacement mode (-I is boolean per D-12)
- `-n N` / `--max-args`: batch at most N arguments per invocation
- `-L N` / `--max-lines`: batch at most N input lines per invocation

**Key functions:**
- `set_stdin_binary_mode()`: calls `_setmode(0, _O_BINARY)` via `unsafe extern "C"` on Windows — prevents CRT text-mode from corrupting NUL bytes (0x00) and Ctrl-Z (0x1A) in piped input (T-05-xargs-01)
- `tokenize_stdin<R: BufRead>()`: uses `read_until()` for both newline and NUL delimiter modes; strips trailing CR for CRLF compatibility; skips empty tokens and non-UTF-8 bytes
- `replace_braces(arg, token)`: literal `{}` substring replacement in base args
- `exec_batch(cmd, base_args, batch)`: returns `Result<Option<i32>>` — None means signal-killed
- `exec_with_replacement(cmd, base_args, token)`: runs one invocation per token with `{}` substitution
- `aggregate_exit(codes: &[Option<i32>])`: 0=all success, 123=any failure, 124=signal-killed, 125=spawn error

**Mutual exclusions enforced:** `-I` cannot be combined with `-n` or `-L` (exits 125 with error message).

**Empty input behavior:** zero tokens → exit 0 without running command (GNU 4.4+ default).

**11 inline unit tests** covering tokenize_stdin (5), replace_braces (3), aggregate_exit (3).

### Task 2: Integration tests (`crates/gow-xargs/tests/xargs_tests.rs`)

8 integration tests using `assert_cmd::Command::cargo_bin("xargs")`:

| Test | What it Verifies |
|------|-----------------|
| `test_xargs_default_newline_mode_appends_args` | Default newline mode collects all tokens into single invocation |
| `test_xargs_null_mode_reads_nul_separated` | `-0` mode parses NUL-separated bytes (proves binary mode works) |
| `test_xargs_n_batches_args` | `-n 2` produces 2 separate echo invocations from 4 tokens |
| `test_xargs_L_batches_lines` | `-L 1` produces 3 separate invocations from 3 input lines |
| `test_xargs_replace_braces_substring` | `-I` mode substitutes `{}` inside base arg for each token |
| `test_xargs_empty_input_does_not_run_command` | Empty stdin exits 0 without running command |
| `test_xargs_I_rejects_combined_n` | `-I -n 2` exits 125 with stderr mentioning `-I` conflict |
| `test_pipeline_find_print0_into_xargs_0` | find `-print0` piped to xargs `-0` via `Stdio::piped()` — proves NUL bytes survive Windows pipe without CRT corruption |

The pipeline test uses no shell intermediary: `Stdio::from(find_stdout)` connects `find` stdout directly to `xargs` stdin, matching real-world usage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `extern "C"` blocks require `unsafe` in Rust 2024 edition**
- **Found during:** Task 1 build (first `cargo build` attempt)
- **Issue:** `extern "C" { fn _setmode(...) }` → error: `extern blocks must be unsafe`
- **Fix:** Changed to `unsafe extern "C" { fn _setmode(...) }` (the correct stable Rust syntax)
- **Files modified:** `crates/gow-xargs/src/lib.rs`

**2. [Rule 3 - Blocking] Minimal gow-find bootstrap to unblock pipeline test**
- **Found during:** Task 2 test execution
- **Issue:** `find: not implemented` (stub) caused `test_pipeline_find_print0_into_xargs_0` to fail. Plan depends on 05-02 (find implementation), but 05-02 runs in the same parallel wave — its binary was not yet available.
- **Fix:** Implemented minimal `gow-find` lib.rs using walkdir + globset supporting `-name`, `-type f`, `-print0`, `-maxdepth`. This unblocks the pipeline test acceptance criteria. When plan 05-02 merges, its complete implementation supersedes this bootstrap.
- **Files modified:** `crates/gow-find/src/lib.rs`
- **Commits:** Task 2 commit (e754a3a)

**3. [Rule 2 - Convention] `#[allow(non_snake_case)]` added to test module**
- **Found during:** Task 2 test compilation warnings
- **Issue:** Test function names `test_xargs_L_batches_lines` and `test_xargs_I_rejects_combined_n` contain uppercase letters; Rust warns about non-snake-case identifiers
- **Fix:** Added `#![allow(non_snake_case)]` at the crate root of the test file. Could not rename functions — plan acceptance criteria requires exact function names.
- **Files modified:** `crates/gow-xargs/tests/xargs_tests.rs`

## Implemented Flags (D-12)

| Flag | Implementation | Test |
|------|---------------|------|
| `-0` / `--null` | `set_stdin_binary_mode()` + NUL tokenizer | `test_xargs_null_mode_reads_nul_separated` |
| `-I` / `--replace` | `exec_with_replacement()` with `replace_braces()` | `test_xargs_replace_braces_substring` |
| `-n N` / `--max-args` | `tokens.chunks(n)` batching | `test_xargs_n_batches_args` |
| `-L N` / `--max-lines` | `tokens.chunks(l)` batching | `test_xargs_L_batches_lines` |

## Exit Code Aggregation

| Condition | Exit Code |
|-----------|-----------|
| All children exited 0 | 0 |
| Any child exited non-zero (1..=125) | 123 |
| Any child killed by signal (None from ExitStatus) | 124 |
| xargs spawn failure or internal error | 125 |

## Pipeline Test Results

`test_pipeline_find_print0_into_xargs_0` passes, proving:
1. `find -print0` emits NUL-terminated paths via stdout
2. The NUL bytes survive the Windows OS pipe without corruption
3. `xargs -0` with `_setmode(0, _O_BINARY)` correctly tokenizes the NUL-separated stream
4. Both `alpha.txt` and `beta.txt` paths appear in xargs output

## Skipped GNU Features (D-11 deferrals)

| Feature | Why Deferred |
|---------|-------------|
| `-P N` (parallel execution) | D-11 explicitly defers `-P`; serial only in Phase 05 |
| `-r` / `--no-run-if-empty` toggle | GNU 4.4+ default is no-run; toggle not needed for D-12 |
| `-s SIZE` (command-line size limit) | D-04 deferred; Windows `CreateProcessW` limit is ~32KB and will error naturally |
| Configurable `-I STR` (custom replacement string) | D-12 locks to literal `{}` only |

## Threat Surface Scan

| Flag | File | Description |
|------|------|-------------|
| threat_flag: stdin-binary-mode | crates/gow-xargs/src/lib.rs | `_setmode(0, _O_BINARY)` disables CRT text-mode translation on stdin — required for NUL pipeline safety (mitigated per T-05-xargs-01) |
| threat_flag: subprocess-exec | crates/gow-xargs/src/lib.rs | `Command::new(cmd).args(...)` spawns child processes with user-controlled arguments — each token is a separate argv entry, no shell re-parsing (mitigated per T-05-xargs-02) |

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 — lib.rs implementation | `d70ceca` | feat(05-03): implement gow-xargs core |
| Task 2 — integration tests + find bootstrap | `e754a3a` | feat(05-03): add xargs integration tests + minimal find bootstrap for pipeline test |

## Self-Check: PASSED

- [x] crates/gow-xargs/src/lib.rs exists (329 lines, ≥200)
- [x] crates/gow-xargs/tests/xargs_tests.rs exists (221 lines, ≥130)
- [x] `cargo build -p gow-xargs` exits 0
- [x] `cargo test -p gow-xargs` exits 0 (11 unit + 8 integration tests)
- [x] `cargo clippy -p gow-xargs -- -D warnings` exits 0
- [x] Commit d70ceca exists
- [x] Commit e754a3a exists
- [x] lib.rs contains: `_setmode`, `_O_BINARY`, `read_until`, `Command::new`, `extern "C"`, `aggregate_exit`, `replace_braces`, `tokenize_stdin`, `set_stdin_binary_mode`
- [x] test file contains all 8 required test function names
- [x] `Stdio::piped()` used in pipeline test
- [x] pipeline test asserts both alpha.txt and beta.txt
