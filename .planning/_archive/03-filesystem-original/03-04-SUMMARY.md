---
phase: 03-filesystem
plan: 04
subsystem: gow-chmod (FILE-10)
tags: [phase3, chmod, permission-model, wave-1, walkdir]
dependency_graph:
  requires:
    - "crates/gow-chmod stub (Plan 03-01 — workspace member + walkdir dep + build.rs + stub lib.rs)"
    - "gow_core::args::parse_gnu (Phase 1)"
    - "gow_core::path::try_convert_msys_path (Phase 1)"
  provides:
    - "pub fn uu_chmod::parse_mode(&str) -> Result<ReadOnlyTarget, String>"
    - "pub enum uu_chmod::ReadOnlyTarget { SetReadOnly, ClearReadOnly, NoOpKeepCurrent }"
    - "chmod.exe — real binary satisfying FILE-10"
  affects:
    - "ROADMAP Phase 3 progress: FILE-10 chmod delivered"
    - "First walkdir-using utility in Phase 3 (proves Wave 0 dep graph works)"
tech_stack:
  added: []
  patterns:
    - "Mode-parser clause loop: state-machine scan over a split(',')-iterated string (new Phase 3 pattern; analog to gow-echo short-flag char loop)"
    - "Dash-mode rescue: preprocess argv before clap to insert `--` ahead of ambiguous tokens like `-w`"
    - "walkdir::WalkDir with follow_links(false) + sort_by_file_name (first use; Shape 1 from RESEARCH Pattern 6)"
    - "apply_readonly + NoOpKeepCurrent three-way enum gives the caller a tri-state (set / clear / skip) without needing a boolean + Option pair"
key_files:
  created:
    - path: "crates/gow-chmod/tests/integration.rs"
      purpose: "16 integration tests driving chmod.exe through assert_cmd; every assertion verifies the actual FILE_ATTRIBUTE_READONLY bit via std::fs::metadata"
  modified:
    - path: "crates/gow-chmod/src/lib.rs"
      change: "Replaced stub uumain with full implementation — ReadOnlyTarget enum + parse_mode (octal + symbolic) + parse_symbolic_clause + apply_readonly + uu_app clap builder + uumain with -R walkdir recursion + looks_like_dash_mode + preprocess_dash_modes + 16 unit tests. 380 lines total."
decisions:
  - "D-32 owner-write-bit-only mapping honored: non-u/non-a symbolic clauses and non-w perm bits are silently no-op; other mode bits (g,o,x,X,s,t) do not produce warnings"
  - "Pre-process argv to rewrite ambiguous dash-modes (Rule 1 fix): chmod -w FILE would otherwise fail clap validation because clap sees -w as an unknown flag. Solution is an argv pre-pass that inserts `--` before the first non-flag token matching [-+=][rwxXst]+"
  - "ReadOnlyTarget::NoOpKeepCurrent used to represent clauses that say nothing about owner-write (e.g. g+w, +x). apply_readonly skips the metadata call entirely in that case — avoids a redundant read+write when nothing would change"
  - "walkdir::WalkDir::follow_links(false) explicitly set to mitigate T-03-15 (chmod -R through a symlinked dir); chosen over follow_links default (already false) for self-documenting code"
  - "-c / --changes maps to verbose identically — noted in help text as 'partial support on Windows'. No way to implement 'only-on-change' without pre-reading the RO bit, which is an extra syscall per file for marginal UX benefit"
metrics:
  duration_minutes: 4
  completed_date: "2026-04-21"
  tasks_completed: 2
  files_created: 1
  files_modified: 1
  tests_added: 32
  test_count_before_plan: 289
  test_count_after_plan: 321
---

# Phase 03 Plan 04: gow-chmod (FILE-10) Summary

Replaces the Plan 03-01 stub with a full GNU-chmod-compatible `chmod.exe` that maps POSIX mode strings to the Windows `FILE_ATTRIBUTE_READONLY` attribute per D-32. Parses both octal (`644`, `0444`, `777`) and symbolic (`u+w`, `-w`, `=r`, `a-w`, comma-chained) forms. Supports `-R` via `walkdir::WalkDir` with `follow_links(false)` so recursion cannot escape a symlink boundary. 32 tests (16 unit + 16 integration) all green; clippy clean.

## What Was Delivered

### Task 1 — Mode parser + apply_readonly + uumain (RED commit `39a1962`, GREEN commit `da205a1`)

- **`ReadOnlyTarget` enum** — three states (`SetReadOnly`, `ClearReadOnly`, `NoOpKeepCurrent`). NoOp lets the caller skip the metadata read+write entirely when a symbolic clause says nothing about owner write.
- **`parse_mode(&str) -> Result<ReadOnlyTarget, String>`** — top-level entry that dispatches on first char. Empty string is an error.
  - **Octal path (`parse_octal`)**: strips leading `0`s, parses base-8, extracts `n & 0o200`. Max 4 digits, but parser accepts any valid base-8 number since trailing garbage is rejected by `u32::from_str_radix`.
  - **Symbolic path (`parse_symbolic_clause`)**: scans `[ugoa]*[+-=][rwxXst]*` per clause. Recognizes `u` and `a` as owner-affecting; `g`/`o` clauses are no-op. Unknown permission characters are an error. Multi-clause strings (`u-w,u+w`) are folded left-to-right; NoOp clauses are skipped so they don't clobber earlier decisions.
- **`apply_readonly(&Path, ReadOnlyTarget, verbose)`** — wraps `std::fs::metadata` + `Permissions::set_readonly` + `std::fs::set_permissions`. Honors the `-v` flag with messages like `mode of '<path>' changed to read-only`. NoOp prints `mode of '<path>' retained` under `-v`.
- **`uumain`** — builds clap command, extracts `-R`/`-v`/`-c`/`-f`, takes operands[0] as mode and operands[1..] as files. Each file is MSYS-normalized, then either walked (if `-R` + directory) or applied directly. Per-file errors accumulate into `exit_code = 1`; the loop never early-returns.
- **16 unit tests** in `crates/gow-chmod/src/lib.rs` — octal, 0-prefixed octal, invalid octal, symbolic `+w`/`-w`/`=r`/`=rw`, owner vs group-only, x-bit noop, empty error, multi-clause override.

### Task 2 — Integration tests (commit `4da445d`)

`crates/gow-chmod/tests/integration.rs`: 16 tests spawn the real `chmod.exe` via `assert_cmd::Command::cargo_bin` and verify the actual `FILE_ATTRIBUTE_READONLY` state via `std::fs::metadata(path).permissions().readonly()` after each invocation.

| Test | Scenario | Assertion |
|------|----------|-----------|
| `test_chmod_644_writable` | octal 644 on RO fixture | becomes writable |
| `test_chmod_444_readonly` | octal 444 on writable fixture | becomes RO |
| `test_chmod_0644_writable` | 4-digit octal | writable |
| `test_chmod_plus_w_clears` | `+w` on RO | writable |
| `test_chmod_minus_w_sets` | `-w` on writable | RO |
| `test_chmod_u_equals_r_sets_ro` | `u=r` | RO |
| `test_chmod_u_equals_rw_clears` | `u=rw` | writable |
| `test_chmod_g_only_is_noop` | `g+w` on writable | unchanged |
| `test_chmod_x_bit_noop` | `+x` on writable | unchanged |
| `test_chmod_recursive` | `-R 644` on dir with 2 RO files | both writable |
| `test_chmod_nonexistent_file` | `644 no-such` | exit 1 + GNU error |
| `test_chmod_invalid_mode_exits_1` | `xyz file` | exit 1 + "invalid" |
| `test_chmod_missing_operand` | no args | exit 1 + "missing operand" |
| `test_chmod_verbose_prints` | `-v 644 file` | stdout "mode of … writable" |
| `test_chmod_bad_flag_exits_1` | `--invalid-flag-xyz` | exit 1 (D-02) |
| `test_chmod_partial_failure_continues` | one missing + one valid | exit 1, valid processed |

## Verification Results

```
cargo test -p gow-chmod --lib              -> 16 passed, 0 failed
cargo test -p gow-chmod                    -> 16 + 16 = 32 passed, 0 failed
cargo clippy -p gow-chmod --all-targets -- -D warnings -> OK
```

FILE-10 spot check (plan verification §3):

```
$ ./target/x86_64-pc-windows-msvc/debug/chmod.exe 444 t.txt && attrib t.txt
A    R               ...\t.txt
$ ./target/x86_64-pc-windows-msvc/debug/chmod.exe 644 t.txt && attrib t.txt
A                    ...\t.txt
```

`R` flag appears after `chmod 444` and disappears after `chmod 644` — confirmed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] `chmod -w FILE` rejected by clap as unknown flag**

- **Found during:** Task 2 first integration-test run (15/16 passing; `test_chmod_minus_w_sets` failed).
- **Issue:** Clap treats any `-x` token as a potential short flag and errors on unknown ones. GNU chmod accepts `-w`, `+x`, `=r` etc. as the first positional (mode) argument even when they begin with `-`. The plan's SECTION C `uu_app` definition did not account for this — `trailing_var_arg(true)` on the operands list is not sufficient because clap parses flags before positional arg collection begins.
- **Fix:** Added two helpers to `crates/gow-chmod/src/lib.rs`:
  - `looks_like_dash_mode(s: &str) -> bool` — matches `[-+=][rwxXst]+` exclusively.
  - `preprocess_dash_modes(Vec<OsString>) -> Vec<OsString>` — scans argv; when the first non-flag token matches the dash-mode pattern and is not a recognized chmod flag (`-R`, `-v`, `-c`, `-f`, `--recursive`, `--verbose`, `--changes`, `--silent`, `--quiet`, `--help`, `--version`), insert a `--` ahead of it so clap treats everything from that point as a positional operand. Respects an explicit `--` already provided by the user.
  - `uumain` now collects args into a Vec, runs the pre-processor, and passes the rewritten slice to `parse_gnu`.
- **Files modified:** `crates/gow-chmod/src/lib.rs` (+67 lines)
- **Commit:** `cd78239`

### Authentication Gates

None.

### Threat Model Verification

Both threats from the plan's `<threat_model>` are mitigated exactly as specified:

- **T-03-04 (ACL widening):** `Permissions::set_readonly` on Windows only toggles `FILE_ATTRIBUTE_READONLY`; it does NOT touch the DACL. Rust std source (`library/std/src/sys/pal/windows/fs.rs`) confirms. No code in this plan touches DACLs.
- **T-03-15 (symlink recursion escape):** `walkdir::WalkDir::new(path).follow_links(false)` on the recursion path in `uumain`. Documented inline in the source comment.

## Acceptance Criteria Verification

| Criterion | Plan target | Actual |
|-----------|-------------|--------|
| `pub fn parse_mode` count | 1 | 1 |
| `pub enum ReadOnlyTarget` count | 1 | 1 |
| `fn apply_readonly` count | 1 | 1 |
| `walkdir::WalkDir` count | >= 1 | 2 |
| `try_convert_msys_path` count | 1 | 1 |
| `set_readonly` count | >= 1 | 3 |
| "not yet implemented" count | 0 | 0 |
| `wc -l` lib.rs | >= 200 | 380 |
| unit tests | >= 15 | 16 |
| integration test count | >= 14 | 16 |
| `set_readonly`/`is_readonly` in tests | >= 3 | 17 |
| `assert_cmd`/`cargo_bin` in tests | >= 1 | 2 |
| `-R` integration test | >= 1 | 1 |
| clippy (-D warnings) | clean | clean |

## TDD Gate Compliance

- RED commit: `39a1962` — `test(03-04): add failing tests for chmod mode parser (RED)` (14 of 16 unit tests failing against the stub)
- GREEN commit: `da205a1` — `feat(03-04): implement chmod mode parser + apply_readonly + uumain with -R (GREEN)` (all 16 unit tests passing)
- Integration tests discovered a Rule-1 bug (dash-mode handling); fix committed as `cd78239` before the integration test commit `4da445d` so the tests landed green.

## Self-Check: PASSED

- **Files created:**
  - `crates/gow-chmod/tests/integration.rs` — present (202 lines, 16 tests)
- **Files modified:**
  - `crates/gow-chmod/src/lib.rs` — present (380 lines, 16 unit tests, 0 "not yet implemented" strings)
- **Commits verified present on current branch:**
  - `39a1962` test(03-04): add failing tests for chmod mode parser (RED)
  - `da205a1` feat(03-04): implement chmod mode parser + apply_readonly + uumain with -R (GREEN)
  - `cd78239` fix(03-04): allow dash-prefixed mode strings (chmod -w FILE)
  - `4da445d` test(03-04): add integration tests for chmod RO-bit semantics
- **Functional verification:**
  - `cargo test -p gow-chmod` → 32 passed / 0 failed
  - `cargo clippy -p gow-chmod --all-targets -- -D warnings` → OK
  - `chmod.exe 444 && attrib` shows `R`; `chmod.exe 644 && attrib` clears `R` — FILE-10 end-to-end
