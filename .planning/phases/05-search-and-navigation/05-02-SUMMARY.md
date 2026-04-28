---
phase: "05-search-and-navigation"
plan: "02"
subsystem: "search-and-navigation"
tags: [find, predicates, walkdir, globset, exec, print0, gnu-compat]
dependency_graph:
  requires: [gow-find-scaffold]
  provides: [gow-find-implementation, find-predicates, find-exec, find-print0]
  affects:
    - crates/gow-find/src/lib.rs
    - crates/gow-find/tests/find_tests.rs
tech_stack:
  added: []
  patterns:
    - GlobBuilder::case_insensitive()/literal_separator(false) for -name/-iname
    - WalkDir::new().min_depth().max_depth().follow_links(false) for depth control
    - parse_gnu() + normalize_find_args() for single-dash long flag normalization
    - std::process::Command (CreateProcessW, no shell) for -exec
    - _setmode(1, _O_BINARY) via unsafe extern "C" for -print0 binary stdout
    - allow_hyphen_values=true on clap args accepting -N values
key_files:
  created:
    - crates/gow-find/tests/find_tests.rs
  modified:
    - crates/gow-find/src/lib.rs
decisions:
  - "normalize_find_args() rewrites single-dash GNU flags (-name, -type, etc.) to double-dash before clap parses — parse_gnu() does not natively handle single-dash long flags"
  - "allow_hyphen_values=true added to -mtime/-atime/-ctime/-size so '-N' values (e.g. -mtime -1) are accepted by clap without being treated as unknown flags"
  - "If both -name and -iname are given, -iname wins (GNU behavior) — documented in code"
  - "unsafe extern 'C' block for _setmode (Rust 2024 edition requires unsafe on extern block)"
metrics:
  duration_seconds: 295
  completed_date: "2026-04-28"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 1
---

# Phase 05 Plan 02: gow-find Implementation — Summary

**One-liner:** Full GNU find implementation with -name/-iname/-type/-size/-mtime/-atime/-ctime/-maxdepth/-mindepth/-print0/-exec, 15 unit tests and 13 integration tests all passing.

## What Was Built

### Task 1: gow-find core (lib.rs — 607 lines)

Complete replacement of the stub with a real GNU-compatible `find` implementation:

**Cli struct** (clap derive, double-dash flags via normalize_find_args):
- `paths: Vec<PathBuf>` — positional paths, default "."
- `--name` / `--iname` — case-sensitive/insensitive glob match (D-01)
- `--type f|d|l` — file/directory/symlink filter (D-03)
- `--size +N/-N/N[c|k|M|G]` — size filter with unit suffixes (D-03)
- `--mtime` / `--atime` / `--ctime +N/-N/N` — time filter in days (D-03/D-04)
- `--maxdepth N` / `--mindepth N` — depth control (D-03)
- `--print0` — NUL-delimited binary stdout (RESEARCH.md Pattern 3)
- `--exec CMD ARGS... ;` — execute command per match (D-05/D-06)

**Key implementation decisions:**

1. `normalize_find_args()` — Rewrites GNU single-dash long flags to double-dash before clap parsing. GNU find uses `-name`, not `--name`. Since `parse_gnu()` doesn't handle this, we normalize in `uumain` before calling clap. The literal `;` terminator and path positionals are left untouched.

2. `build_name_matcher()` uses `GlobBuilder::new(pattern).case_insensitive(ci).literal_separator(false).build()?.compile_matcher()`. The `literal_separator(false)` is correct per POSIX — `*` in a basename pattern should match any character.

3. `match_glob_basename()` matches against `entry.file_name()` (basename only), NOT `entry.path()`. This is the GNU `find -name` semantic per RESEARCH.md Pitfall 3.

4. `exec_for_entry()` uses `std::process::Command::new(cmd).args(&substituted_args).status()` — no shell intermediary. Each argument is passed as a separate string to `CreateProcessW`. This is the GOW #209 fix.

5. `set_stdout_binary_mode()` calls `_setmode(1, _O_BINARY)` via `unsafe extern "C"` (CRT function, not Win32 API). Called once before any stdout writes when `-print0` is active. Per T-05-find-05 mitigation.

6. `allow_hyphen_values = true` added to `--mtime`/`--atime`/`--ctime`/`--size` clap args so values like `-1`, `-7`, `-1k` are accepted without clap treating them as unknown flags.

### Task 2: Integration tests (find_tests.rs — 300 lines)

13 integration tests using `assert_cmd` + `predicates` + `tempfile`:

| Test | Predicate Covered |
|------|-------------------|
| `test_name_matches_basename_only` | -name glob, basename semantics |
| `test_name_is_case_sensitive` | -name case-sensitivity (D-01) |
| `test_iname_is_case_insensitive` | -iname case-insensitivity (D-01) |
| `test_type_filter_files` | -type f |
| `test_type_filter_directories` | -type d |
| `test_size_greater_than` | -size +1k |
| `test_maxdepth_zero_lists_root_only` | -maxdepth 0 |
| `test_maxdepth_one_skips_subdir_contents` | -maxdepth 1 |
| `test_mtime_recent_files` | -mtime -1 (recently created file) |
| `test_exec_runs_command_per_match` | -exec cmd {} ; |
| `test_exec_handles_paths_with_spaces` | GOW #209 regression — spaces in path |
| `test_print0_emits_null_separated_paths` | -print0 binary NUL output |
| `test_multiple_predicates_and_together` | AND semantics across predicates |

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build -p gow-find` | Exit 0 |
| `cargo test -p gow-find --lib` | 15/15 passing |
| `cargo test -p gow-find` | 28/28 passing (15 unit + 13 integration) |
| `cargo clippy -p gow-find -- -D warnings` | Exit 0, no warnings |
| `cargo run -p gow-find -- . -maxdepth 0` | Prints "." only |
| `cargo run -p gow-find -- . -name "*.toml" -type f -maxdepth 1` | Prints `.\Cargo.toml` |
| GlobBuilder::new present | 1 occurrence |
| WalkDir::new present | 1 occurrence |
| Command::new present | 1 occurrence |
| _setmode present | 3 occurrences |
| follow_links(false) present | 1 occurrence |
| literal_separator(false) present | 1 occurrence |
| File length ≥ 250 lines | 607 lines |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] extern block required `unsafe` prefix (Rust 2024 edition)**
- **Found during:** Task 1 initial build
- **Issue:** Rust 2024 edition requires `unsafe extern "C"` for FFI declarations. The plan's code snippet used `extern "C"` without `unsafe`.
- **Fix:** Changed to `unsafe extern "C" { fn _setmode(...) }` in `set_stdout_binary_mode()`
- **Files modified:** `crates/gow-find/src/lib.rs`
- **Commit:** `5cd247d`

**2. [Rule 1 - Bug] clap rejected `-N` values for time/size predicates**
- **Found during:** Task 2 integration test run (test_mtime_recent_files FAILED)
- **Issue:** clap 4 treats values starting with `-` as potential flags unless `allow_hyphen_values = true` is set on the argument. `-mtime -1` caused clap to reject `-1` as "unexpected argument".
- **Fix:** Added `allow_hyphen_values = true` to `--mtime`, `--atime`, `--ctime`, `--size` clap arg annotations.
- **Files modified:** `crates/gow-find/src/lib.rs`
- **Commit:** `699a0d9`

**3. [Rule 1 - Bug] `ref matcher` binding mode error in Rust 2024**
- **Found during:** Task 1 initial build
- **Issue:** `if let Some(ref matcher) = name_matcher` — Rust 2024 edition disallows explicit `ref` binding modifier when implicitly borrowing.
- **Fix:** Changed to `if let Some(matcher) = name_matcher` (ref removed; implicit borrow is correct here).
- **Files modified:** `crates/gow-find/src/lib.rs`
- **Commit:** `5cd247d`

## GOW Issue Regressions

| Issue | Test | Status |
|-------|------|--------|
| GOW #208 (exec support) | `test_exec_runs_command_per_match` | Passing |
| GOW #209 (paths with spaces in exec) | `test_exec_handles_paths_with_spaces` | Passing |

## Known Stubs

None. This plan delivers a complete implementation of all predicates locked in CONTEXT.md D-01 through D-06.

## Threat Surface Scan

The following security-relevant surfaces were introduced and match the plan's threat model:

| Surface | File | Threat Model Entry |
|---------|------|--------------------|
| `-exec` command execution via CreateProcessW | lib.rs `exec_for_entry()` | T-05-find-01: std::process::Command, no shell intermediary |
| `WalkDir::follow_links(false)` | lib.rs `run()` | T-05-find-02: symlink loop prevention |
| `GlobBuilder::build()` returns Err on invalid glob | lib.rs `build_name_matcher()` | T-05-find-03: graceful glob error handling |
| `_setmode(1, _O_BINARY)` for -print0 stdout | lib.rs `set_stdout_binary_mode()` | T-05-find-05: null-byte pipeline safety |

No new threat surfaces beyond those documented in the plan's threat model.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 — lib.rs implementation | `5cd247d` | feat(05-02): implement gow-find core — Cli, predicates, WalkDir, -exec, -print0 |
| Task 1 fix — allow_hyphen_values | `699a0d9` | fix(05-02): allow_hyphen_values on -mtime/-atime/-ctime/-size for negative specs |
| Task 2 — integration tests | `32b9f0c` | test(05-02): add 13 integration tests for gow-find predicates and actions |

## Self-Check: PASSED

- [x] `crates/gow-find/src/lib.rs` exists (607 lines)
- [x] `crates/gow-find/tests/find_tests.rs` exists (300 lines, 13 tests)
- [x] Commit 5cd247d exists
- [x] Commit 32b9f0c exists
- [x] Commit 699a0d9 exists
- [x] `cargo test -p gow-find` exits 0 (28 tests passing)
- [x] `cargo clippy -p gow-find -- -D warnings` exits 0
