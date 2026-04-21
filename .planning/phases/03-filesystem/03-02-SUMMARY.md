---
phase: 03-filesystem
plan: 02
subsystem: gow-cat
tags: [phase3, cat, byte-stream, wave-1, file-01]
dependency_graph:
  requires:
    - "gow-core::init (UTF-8 console + VT100)"
    - "gow-core::args::parse_gnu (GNU exit-code + option permutation)"
    - "gow-core::path::try_convert_msys_path (D-26 per-operand MSYS)"
    - "Phase 3 Plan 03-01 (gow-cat stub crate scaffold)"
  provides:
    - "crates/gow-cat: real FILE-01 implementation (cat.exe)"
    - "Reusable uumain+operand-loop shape for head, chmod (Wave 1), dos2unix, unix2dos (Wave 2)"
    - "parse_gnu help/version exit-0 behavior (side-effect fix benefits all 14 utilities)"
  affects:
    - "gow-core::args::parse_gnu — DisplayHelp/DisplayVersion now exit 0"
tech_stack:
  added:
    - "clap 4.6 (already workspace) — ArgAction::SetTrue per flag"
  patterns:
    - "BufReader::read_until(b'\\n') byte-safe line iteration (D-48)"
    - "visualize_byte state-machine (recursive on high-bit via M- prefix)"
    - "CatState struct: line_number + prev_blank persist across operands"
    - "Per-operand MSYS path conversion via gow_core::path::try_convert_msys_path"
    - "Per-file error format 'cat: {path}: {err}' → exit 1, continue to next operand"
key_files:
  created:
    - path: "crates/gow-cat/tests/integration.rs"
      purpose: "19 integration tests covering all 7 flags, UTF-8 Korean, CP949 passthrough, multi-operand + dash, error paths"
  modified:
    - path: "crates/gow-cat/src/lib.rs"
      change: "Replaced 22-line stub with 372-line real implementation: uumain + uu_app + Opts + visualize_byte + is_blank_line + CatState + cat_reader + 14 unit tests"
    - path: "crates/gow-core/src/args.rs"
      change: "Rule 2 deviation — parse_gnu now handles clap ErrorKind::DisplayHelp and DisplayVersion with exit 0 (GNU convention). All other utilities benefit."
decisions:
  - "Blank line = any-combination of \\n\\r\\t\\ OR nothing (is_blank_line) — matches GNU cat -b/-s semantics without bstr dependency"
  - "visualize_byte uses simple recursion for 0x80-0xFF high-bit: strip top bit, reapply ASCII rules. One function, six branches."
  - "Line numbering counter persists across files per GNU behavior (-n f1 f2 → 1, 2 for 2 single-line files, not 1, 1)"
  - "-A is a true shorthand that sets -v+E+T (plan open question 2 recommendation) — not a separate codepath"
  - "-b overrides -n in Opts::from_matches so cat_reader doesn't double-number (simpler than branching in the loop)"
  - "Per-operand error reporting (not fail-fast) — GNU processes all operands and returns 1 if any failed"
metrics:
  duration_minutes: 3
  completed_date: "2026-04-21"
  tasks_completed: 2
  files_created: 1
  files_modified: 2
  tests_added: 33
  test_count_before: "289 (workspace baseline at end of 03-01)"
  test_count_after: "322 in gow-cat/gow-core combined: 47 gow-core unit + 3 doctest + 14 gow-cat unit + 19 gow-cat integration + others unchanged"
---

# Phase 03 Plan 02: gow-cat (FILE-01) Summary

**One-liner:** GNU `cat` via byte-stream BufReader::read_until(b'\n') with all 7 flags (-n -b -s -v -E -T -A), per-operand MSYS conversion, accumulated line numbering, and Korean UTF-8 / CP949 byte passthrough per D-48.

## What Was Delivered

### Task 1 — Real cat implementation (commit `68603fd`)

- **`crates/gow-cat/src/lib.rs`** — 372 lines replacing the 22-line Plan 03-01 stub:
  - `uu_app()` — clap Command builder with 7 flags + `trailing_var_arg` operands
  - `Opts` struct + `Opts::from_matches` — resolves `-A` → `-vET` and `-b` overrides `-n`
  - `visualize_byte(b, out, opts)` — pure helper encoding per GNU rules:
    - `\n` → emits `$\n` if `-E`, else plain `\n`
    - `\t` → emits `^I` if `-T`, else plain `\t`
    - `0x00..=0x1F` (control) → emits `^X` (b XOR 0x40) if `-v`
    - `0x7F` (DEL) → emits `^?` if `-v`
    - `0x80..=0xFF` (high bit) → emits `M-` then recurses on `b & 0x7F` if `-v`
    - Other bytes → pass-through
  - `is_blank_line(bytes)` — only whitespace or line endings (for `-b` and `-s`)
  - `CatState { line_number, prev_blank }` — persists across operands
  - `cat_reader<R,W>` — loops `read_until(b'\n')` → blank check → squeeze → number → visualize → write
  - `uumain` — stdin if no operands, else iterates operands with `-` → stdin, `path` → `try_convert_msys_path` → `File::open`; per-file `cat: {path}: {err}` on error with `exit_code=1` but continues
  - 14 unit tests (visualize_byte matrix, is_blank_line cases, `-b` override, `-A` shorthand)

### Task 2 — Integration tests + parse_gnu help fix (commit `94abc3b`)

- **`crates/gow-cat/tests/integration.rs`** — 19 `#[test]` functions via `assert_cmd`:
  - `test_cat_passthrough` — plain single file
  - `test_cat_n_line_numbers` — `     1\ta\n` format
  - `test_cat_n_utf8_korean` — byte-exact UTF-8 Korean preservation (**ROADMAP criterion 4**)
  - `test_cat_b_nonblank_only` — blank lines not numbered
  - `test_cat_s_squeeze_blanks` — 4 blank lines → 1
  - `test_cat_v_control_chars` — `\r\x01` → `^M^A`
  - `test_cat_e_dollar` — `$` before `\n`
  - `test_cat_t_tabs` — `\t` → `^I`
  - `test_cat_a_equiv_vet` — `-A` = `-vET`
  - `test_cat_dash_reads_stdin` — explicit `-` operand
  - `test_cat_no_operand_reads_stdin` — default stdin
  - `test_cat_multi_file_concat` — no `==>` headers (cat is silent)
  - `test_cat_multi_file_with_dash` — `f1 - f2` mixes file + stdin + file
  - `test_cat_nonexistent_file` — stderr contains `cat:` and filename, exit 1
  - `test_cat_partial_failure_continues` — `cat no-such good.txt` still emits `good.txt`, exits 1
  - `test_cat_cp949_bytes_passthrough` — `[0xBE, 0xC8, 0xB3, 0xE7]` round-trips unchanged (**D-48**)
  - `test_cat_n_accumulates_across_files` — counter doesn't reset
  - `test_cat_help_does_not_panic` — exits 0
  - `test_cat_bad_flag_exits_1` — D-02 exit code

- **`crates/gow-core/src/args.rs`** — see Deviations below.

## Verification Results

```
cargo test -p gow-cat               -> 33 passed, 0 failed (14 unit + 19 integration + 0 doctest)
cargo test -p gow-core              -> 47 passed, 0 failed + 3 doctests passed
cargo clippy -p gow-cat -p gow-core --all-targets -- -D warnings -> clean

# Spot-checks
./target/x86_64-pc-windows-msvc/debug/cat.exe --help         -> exit 0, prints cat help to stdout
./target/x86_64-pc-windows-msvc/debug/cat.exe --invalid-flag -> exit 1
```

**ROADMAP criterion 4 (cat -n on Korean UTF-8 without mojibake):** SATISFIED by `test_cat_n_utf8_korean` — the test writes `"안녕\n세계\n".as_bytes()` to a fixture, spawns cat -n, and asserts the byte-exact stdout `"     1\t안녕\n     2\t세계\n".as_bytes()`. Because D-48 keeps the entire pipeline at the raw-byte level (BufReader::read_until emits bytes verbatim, write_all preserves bytes verbatim, `gow_core::init` already set UTF-8 codepage via SetConsoleOutputCP 65001 in Phase 1), the Korean renders correctly in both Windows Terminal and ConHost.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Missing Critical Functionality] `parse_gnu` exits 1 on `--help`**

- **Found during:** Task 2 integration test run (`test_cat_help_does_not_panic` failed with exit 1 and help text on stderr).
- **Issue:** `gow_core::args::parse_gnu` routes every clap error (including `ErrorKind::DisplayHelp` and `ErrorKind::DisplayVersion`) through `eprintln!` + `std::process::exit(1)`. clap's `--help` / `--version` are implemented as "errors" that carry the rendered help text — they should exit 0 with output on **stdout**, not exit 1 with output on stderr. The plan's acceptance criterion explicitly required `cat.exe --help` to exit 0, and every other gow utility has the same latent bug.
- **Fix:** Matched on `e.kind()` in the `unwrap_or_else` branch. `ErrorKind::DisplayHelp` and `ErrorKind::DisplayVersion` now `print!` (stdout) and `std::process::exit(0)`. All other error kinds keep the original GNU-style `eprintln!` + `exit(1)` behavior. Strictly additive — no existing behavior changed.
- **Files modified:** `crates/gow-core/src/args.rs` (import `ErrorKind`; replace single exit call with `match e.kind()` block)
- **Commit:** `94abc3b`

**2. [Rule 1 — Bug] unit test function name violated snake_case**

- **Found during:** Task 1 `cargo test -p gow-cat --lib` (warning, not failure).
- **Issue:** Unit test `opts_A_enables_vet` uses uppercase `A` → clippy non_snake_case warning.
- **Fix:** Renamed to `opts_a_shorthand_enables_vet` (describes behavior more precisely anyway).
- **Commit:** `68603fd` (caught before commit)

**3. [Rule 2 — Test coverage gap] Plan's `<action>` block for Task 2 omitted `test_cat_multi_file_with_dash`**

- **Found during:** Task 2 verification while reviewing plan's `<behavior>` list.
- **Issue:** Plan's Task 2 `<behavior>` explicitly requires `test_cat_multi_file_with_dash` (`cat f1 - f2` emits `a\n<stdin>\nb\n`), but the `<action>` code block skips it. Acceptance criterion `>=16 tests` would still pass without it, but a documented required behavior would be untested.
- **Fix:** Added `test_cat_multi_file_with_dash` that writes "MID\n" to stdin and asserts `a\nMID\nb\n`. Counts toward the 19 total.
- **Commit:** `94abc3b`

### Authentication Gates

None.

## Flags Implemented

All 7 required flags (per plan must_haves):

| Flag | Long form | Behavior |
|------|-----------|----------|
| `-n` | `--number` | Number every line (6-space right-aligned + tab) |
| `-b` | `--number-nonblank` | Number non-blank lines only; overrides `-n` |
| `-s` | `--squeeze-blank` | Collapse 2+ consecutive blank lines to 1 |
| `-v` | `--show-nonprinting` | `^X` for control, `^?` for DEL, `M-X` for high bit |
| `-E` | `--show-ends` | Append `$` before each `\n` |
| `-T` | `--show-tabs` | Display `\t` as `^I` |
| `-A` | `--show-all` | Shorthand for `-vET` (per plan open question 2) |

## Test Count

- **14 unit tests** in `crates/gow-cat/src/lib.rs` — visualize_byte matrix, is_blank_line, Opts::from_matches
- **19 integration tests** in `crates/gow-cat/tests/integration.rs` — all flag combos + UTF-8 + CP949 + error paths
- **Total: 33 tests** in gow-cat (plan required ≥ 26)

## Path Forward

- `gow-cat` is the Wave 1 template proven: head and chmod can lift the same `uumain` operand-loop shape
- `gow_core::args::parse_gnu` now produces correct `--help` / `--version` behavior for every utility
- `visualize_byte` can be lifted into gow-core later if dos2unix/unix2dos need similar byte-level encoding (Wave 2)

## Self-Check: PASSED

- **Files created:** `crates/gow-cat/tests/integration.rs` — FOUND
- **Files modified:**
  - `crates/gow-cat/src/lib.rs` (from stub to 372 lines) — FOUND
  - `crates/gow-core/src/args.rs` (help/version fix) — FOUND
- **Commits verified present:**
  - `68603fd` — feat(03-02): implement gow-cat uumain with -n/-b/-s/-v/-E/-T/-A flags — FOUND
  - `94abc3b` — test(03-02): add cat integration tests + fix parse_gnu help/version exit code — FOUND
- **Functional verification:**
  - `cargo test -p gow-cat` → 33 passed / 0 failed
  - `cargo clippy -p gow-cat -p gow-core --all-targets -- -D warnings` → OK
  - `cat.exe --help` → exit 0, stdout contains "cat" and "Number all"
  - `cat.exe --invalid-flag` → exit 1
