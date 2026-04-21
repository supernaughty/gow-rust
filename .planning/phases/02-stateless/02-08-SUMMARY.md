---
phase: 02-stateless
plan: 08
subsystem: text
tags: [wc, bstr, unicode, tdd, wave-3, utf-8, text-counting]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: gow_core::init / args::parse_gnu / path::try_convert_msys_path (inherited unchanged)
  - plan: 02-01
    provides: workspace bstr = { workspace = true } dep, gow-wc stub crate scaffold
provides:
  - crates/gow-wc compiled-clean with real uumain (9 unit tests + 12 integration tests = 21 passing)
  - count_bytes(&[u8]) -> Counts public API (lines, words, bytes, chars)
  - TEXT-03 + ROADMAP success criterion 2 observable (wc -m ≠ wc -c on UTF-8)
  - Threat T-02-08-02 mitigation verified: invalid UTF-8 → U+FFFD, never panic
affects: [02-10 (if phase tracking plan runs), Phase 3+ future text utilities may reuse count_bytes]

# Tech tracking
tech-stack:
  added:
    - bstr 1.12.1 (via workspace; byte-safe char iteration for -m / -w)
  patterns:
    - "Counts struct + count_bytes(&[u8]) pure function — testable independent of I/O"
    - "2-pass column width: collect rows into Vec<(String, Counts)>, compute max-digit width, then print (D-30 Claude's Discretion, matches GNU wc layout)"
    - "bstr::ByteSlice::chars() for -m / -w — yields char (U+FFFD on invalid UTF-8) instead of std::str::from_utf8 panicking"
    - "Per-operand error loop: file-open or read failure prints 'wc: {path}: {err}', sets exit_code=1, continues to next operand"
    - "operand '-' routes to io::stdin().lock(); empty-operand list also routes to stdin but without a trailing filename"
    - "MSYS path pre-convert applied to each file operand via gow_core::path::try_convert_msys_path"

key-files:
  created:
    - crates/gow-wc/build.rs
    - crates/gow-wc/tests/integration.rs
    - .planning/phases/02-stateless/02-08-SUMMARY.md
  modified:
    - crates/gow-wc/Cargo.toml
    - crates/gow-wc/src/lib.rs
    - Cargo.lock

decisions:
  - "Column width = 2-pass max-digit-width (CONTEXT.md Claude's Discretion). Implementation collects rows first, computes `max(all requested counts incl. total).to_string().len()`, then uses `format!(\"{:>width$}\")`."
  - "count_bytes returns all four counts unconditionally; flag selection happens in uumain's print_row (simpler than per-flag counting branches; perf cost negligible vs I/O)."
  - "in_word state machine over bstr chars for -w word boundary — matches GNU wc by operating on Unicode scalar values with char::is_whitespace."

metrics:
  duration: 6m20s          # plan start 00:51:37Z → end 00:57:57Z
  completed: 2026-04-21
---

# Phase 2 Plan 08: gow-wc Summary

**One-liner:** GNU wc re-implemented as a UTF-8-native Rust crate using bstr's panic-safe char iterator, delivering `-l / -w / -c / -m` with Unicode-aware word boundaries and right-aligned multi-file totals — 21 tests pin down TEXT-03 and ROADMAP criterion 2 including the `wc -m` Korean fixture and an invalid-UTF-8 regression guard.

## What Was Built

### Core implementation (`crates/gow-wc/src/lib.rs`)

- `pub struct Counts { lines, words, bytes, chars }` — 4-field counter with `.add(&other)` accumulator.
- `pub fn count_bytes(bytes: &[u8]) -> Counts` — single pass over raw bytes:
  - `bytes` = `input.len()`
  - `lines` = `bytes.iter().filter(|&&b| b == b'\n').count()` (raw byte filter; invalid-UTF-8 safe)
  - `chars` = `bstr::ByteSlice::chars(&input).count()` — each iterator step is one `char`; invalid UTF-8 sequences yield U+FFFD (= 1 char). No panic path.
  - `words` = scan `bstr::chars()`, flip-flop an `in_word` flag using `char::is_whitespace`, increment on whitespace→non-whitespace transition.
- `pub fn uumain<I: IntoIterator<Item=OsString>>(args) -> i32`:
  - clap app with `-l / --lines`, `-w / --words`, `-c / --bytes`, `-m / --chars` + variadic `operands`.
  - If no flag given → default is `-l -w -c` (GNU wc default).
  - Operands iterated: `-` routes to `io::stdin().lock()`; otherwise `try_convert_msys_path` → `File::open` → `BufReader`.
  - Empty operand list reads stdin with no trailing filename.
  - Error handling: per-operand `eprintln!("wc: {path}: {err}")` + sets `exit_code = 1`; next operand still processed.
  - Output: 2-pass column width — collect all rows + total, compute max digit count, then `format!("{:>width$}")` on each. `total` row appended when `rows.len() > 1`.

### Build script (`crates/gow-wc/build.rs`)

Verbatim gow-probe template — embeds Windows application manifest with `ActiveCodePage::Utf8` + `longPathAware::Enabled`. Only runs on Windows (`CARGO_CFG_WINDOWS`).

### Cargo.toml (`crates/gow-wc/Cargo.toml`)

Added to stub:
- `bstr = { workspace = true }` under `[dependencies]`
- `embed-manifest = "1.5"` under `[build-dependencies]`
- `assert_cmd`, `predicates`, `tempfile` (all workspace) under `[dev-dependencies]`
- `description` updated to mention Unicode awareness

### Integration tests (`crates/gow-wc/tests/integration.rs`)

12 `assert_cmd`-driven tests covering 02-VALIDATION.md Dimensions 1 / 2 / 4:

1. `test_default_prints_lines_words_bytes` — `wc simple.txt` → `2 4 20 simple.txt`
2. `test_l_flag_only_lines` — single column output for `-l`
3. `test_c_flag_only_bytes` — `-c` on `"12345"` → `5`
4. `test_w_flag_only_words` — `-w` on `"one two three\n"` → `3`
5. `test_m_flag_utf8_char_count` — `-m` on `"안녕 세상\n"` → `6`
6. `test_c_vs_m_differ_on_utf8` — `"안녕\n"` yields `-c=7` but `-m=3` (ROADMAP criterion 2)
7. `test_multi_file_prints_total_line` — two files → stdout contains `"total"`
8. `test_stdin_with_dash_operand` — `echo "x y z" | wc -` → `1 3 6 -`
9. `test_stdin_no_operand` — `echo "alpha beta" | wc` → `1 2 11` (no filename)
10. `test_missing_file_exits_1` — `/nonexistent` → exit 1 + stderr starts with `"wc:"`
11. `test_bad_flag_exits_1` — `--completely-unknown-xyz` → exit 1
12. `test_invalid_utf8_no_panic_on_m` — `[0xFF, 0xFE, 0x0A]` via `-m` → exit 0, no panic (threat T-02-08-02 regression guard)

### Unit tests (9 in `src/lib.rs`)

Drive `count_bytes` directly: empty input, `hello\n`, `hello world\n`, `line1\nline2\n`, Korean `안녕\n`, Korean-with-space, invalid-UTF-8 no-panic, unterminated last line, `Counts::add` accumulator.

## Deviations from Plan

None — plan executed exactly as written. Plan's unit tests list (7 explicit + 1 extra for `add`) all implemented verbatim. Integration tests match plan's 12-test list 1:1.

No Rule 1 / 2 / 3 auto-fixes were needed. No Rule 4 architectural questions arose.

## Output Samples

From the integration-test run (predicate-match stdout):

- **Single file, default:** `  2  4 20 simple.txt`  (2 lines, 4 words, 20 bytes, right-aligned)
- **-m Korean fixture:** `  6 m.txt`  (scalar count only)
- **-c vs -m on `안녕\n`:** `-c` → `  7 diff.txt`, `-m` → `  3 diff.txt`
- **Multi-file total:** first row `  1 1 2 a.txt`, second `  2 2 4 b.txt`, footer `  3 3 6 total` (actual widths adjusted to max digit count)
- **stdin via dash:** `  1 3 6 -`
- **stdin no operand:** `  1 2 11` (no trailing label)
- **Missing file:** stderr `wc: /this/absolutely/does/not/exist/xyzzy.txt: The system cannot find the path specified. (os error 3)`, exit 1
- **Invalid UTF-8 on -m:** exit 0, no panic — bstr::chars yields U+FFFD for the invalid bytes

## Decisions Made

- **1-pass vs 2-pass column width → 2-pass.** Reason: 2-pass matches GNU wc's dynamic-width behaviour and is still O(N) overall (I/O dominates); simpler mental model (collect → print) than running-width updates. Per D-30 Claude's Discretion and CONTEXT.md "Claude's Discretion" section.
- **count_bytes returns all 4 counts unconditionally.** Flag selection gates output, not counting. Makes the core function pure + trivially testable; bstr::chars iteration cost is negligible vs file I/O. If a future benchmark shows hot-path cost, per-flag branching is a local optimisation that doesn't touch the API.
- **Error handling: continue on failed operand.** Matches GNU wc behaviour — `wc a.txt /missing b.txt` reports counts for a.txt and b.txt, prints one `wc: /missing: ...` line, exits 1. Test `test_missing_file_exits_1` pins this.
- **MSYS path pre-convert before `File::open`.** Follows D-08/D-27 pattern established Phase 1. Path `/c/Users/foo.txt` is auto-translated to `C:\Users\foo.txt`.

## Test Counts

| Category                  | Count | Status |
|---------------------------|-------|--------|
| Unit tests (src/lib.rs)   | 9     | all pass |
| Integration tests         | 12    | all pass |
| Doc tests                 | 0     | n/a |
| **Total**                 | **21** | **all pass** |

`cargo clippy -p gow-wc --all-targets -- -D warnings` → clean.
`cargo test -p gow-wc` → 21 passed, 0 failed.

## TDD Gate Compliance

Plan 02-08 is `type=execute` (not `type: tdd` plan-wide), but Task 1 had `tdd="true"`. Gate sequence:

- **RED gate:** commit `014a90f` `test(02-08): add failing count_bytes tests + Cargo.toml bstr dep + build.rs` — 8 tests fail via `todo!()`.
- **GREEN gate:** commit `49579b6` `feat(02-08): implement wc core + uumain ...` — 9 unit tests pass (added `counts_add_accumulates` as a bonus).
- **REFACTOR gate:** not needed — GREEN code is already clean (one struct, one function, no obvious duplication). Clippy `-D warnings` clean.
- **Integration tests commit:** `7ed5da4` `test(02-08): add 12 integration tests (assert_cmd)` — completes Task 2.

## Threat Register Follow-Up

- **T-02-08-01 (DoS: unbounded read_to_end on huge file):** accepted; not mitigated. `count_reader` reads into `Vec<u8>` unbounded. Future v2 work could stream via `BufRead::read_until(b'\n', ...)` + per-line `count_bytes`; plan's accept disposition stands.
- **T-02-08-02 (invalid-UTF-8 panic):** **mitigated**. `bstr::chars()` replaces `std::str` iteration everywhere. Unit test `count_invalid_utf8_no_panic` + integration test `test_invalid_utf8_no_panic_on_m` both pass; regression guards in place.
- **T-02-08-03 (info disclosure via error path):** accepted; path echoes match GNU wc convention (`wc: {path}: {err}`).

No new surface beyond plan's threat model. No `## Threat Flags` section.

## Commits

| Hash | Type | Message |
|------|------|---------|
| `014a90f` | test | add failing count_bytes tests + Cargo.toml bstr dep + build.rs (RED) |
| `49579b6` | feat | implement wc core + uumain (bstr-based Unicode-aware counts) (GREEN) |
| `7ed5da4` | test | add 12 integration tests (assert_cmd) |

## Known Stubs

None. All features listed in plan are wired end-to-end. `-L` (longest line display width) is explicitly deferred to v2 per D-17c — not a stub, a scoped omission.

## Self-Check: PASSED

- FOUND: `crates/gow-wc/Cargo.toml` — `bstr = { workspace = true }` present under `[dependencies]`
- FOUND: `crates/gow-wc/build.rs` — embeds manifest via embed-manifest on Windows
- FOUND: `crates/gow-wc/src/lib.rs` — `Counts`, `count_bytes`, `uumain`, clap `uu_app()`
- FOUND: `crates/gow-wc/tests/integration.rs` — 12 `#[test]` functions
- FOUND: commit `014a90f` in `git log` (RED)
- FOUND: commit `49579b6` in `git log` (GREEN / feat)
- FOUND: commit `7ed5da4` in `git log` (integration tests)
- VERIFIED: `cargo test -p gow-wc` → 21 passed, 0 failed
- VERIFIED: `cargo clippy -p gow-wc --all-targets -- -D warnings` → clean

All plan acceptance criteria met; no missing items.
