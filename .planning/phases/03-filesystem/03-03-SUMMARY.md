---
phase: 03-filesystem
plan: 03
subsystem: head (byte-stream line/byte prefix)
tags: [phase3, head, byte-stream, wave-1, TEXT-01]
dependency_graph:
  requires:
    - "crates/gow-core (parse_gnu, init, path::try_convert_msys_path — Phase 1)"
    - "crates/gow-head stub + build.rs (Wave 0 / plan 03-01)"
  provides:
    - "head.exe real binary — GNU head compatible for -n / -c / -q / -v + numeric shorthand"
    - "uu_head::{uumain, read_n_lines, read_n_bytes, expand_numeric_shorthand} library API"
    - "Reference pattern for byte-stream utilities (Pattern B/C/D/E/H — will be reused by tail)"
  affects:
    - "TEXT-01 requirement now implemented"
    - "Wave 1 (cat/chmod/head) companion — head was the simplest of the three"
tech_stack:
  added: []
  patterns:
    - "Numeric shorthand via argv pre-parse (expand_numeric_shorthand) — new reusable primitive for utilities that need GNU `-N` historical form"
    - "BufReader::take(n) + io::copy for byte-exact N-byte output (mid-multibyte split is a feature, not a bug — D-48)"
    - "BufRead::read_until(b'\\n') for raw-byte line slicing (does not decode UTF-8 — preserves arbitrary bytes)"
    - "Header emission rule: `!quiet && (multi || verbose)` — mirrors GNU head's boolean"
key_files:
  created:
    - path: "crates/gow-head/tests/integration.rs"
      purpose: "18 integration tests: flag matrix, shorthand, headers, stdin, UTF-8, errors, mid-multibyte -c split"
    - path: ".planning/phases/03-filesystem/deferred-items.md"
      purpose: "Logs one out-of-scope pre-existing gow-core unused-import observed during verification"
  modified:
    - path: "crates/gow-head/src/lib.rs"
      change: "Replaced 22-line stub with 388-line real implementation: uu_app, Mode enum, expand_numeric_shorthand, parse_num, read_n_lines, read_n_bytes, uumain; 14 unit tests"
decisions:
  - "Numeric shorthand implemented via pre-parse (not via clap value_parser) because clap treats bare `-<digits>` as unknown short flag even with allow_negative_numbers(true). Pre-parse rewrite is simpler and matches Phase 1 D-05 intent."
  - "-c uses raw `io::copy` on a `BufReader::take(n)` — zero-copy byte transfer; no UTF-8 decode; preserves exact GNU byte-boundary semantics including mid-multibyte split (D-48)."
  - "GNU suffixes (k/M/G on -n/-c values) intentionally deferred to a future plan — parse_num rejects them with exit 1 GNU-style error. Spec'd as v1 scope in plan."
  - "Header separator uses `writeln!()` (blank line) before second-and-later headers to match GNU format: `==> f1 <==\\n<content>\\n==> f2 <==`."
metrics:
  duration_minutes: 4
  completed_date: "2026-04-21"
  tasks_completed: 2
  files_created: 2
  files_modified: 1
  tests_added: 32
  unit_tests: 14
  integration_tests: 18
---

# Phase 03 Plan 03: gow-head Summary

GNU head implemented as a byte-stream, line-or-byte prefix utility. Replaces the Wave 0 stub with a real binary supporting `-n NUM` / `-c NUM` / `-q` / `-v` and GNU numeric shorthand (`head -5 file`). 32 tests green (14 unit + 18 integration), clippy clean on gow-head.

## What Was Delivered

### Task 1 — Real `uu_head::uumain` + readers + shorthand pre-parse (commit `d90bc87`)

`crates/gow-head/src/lib.rs` grew from a 22-line stub to 388 lines with:

| Item | Purpose |
|------|---------|
| `fn uu_app() -> Command` | clap command with `-n`/`--lines` (default 10), `-c`/`--bytes`, `-q`/`--quiet`/`--silent`, `-v`/`--verbose`, and trailing `operands` |
| `enum Mode { Lines(u64), Bytes(u64) }` | Single counting mode picked once per run |
| `fn expand_numeric_shorthand(&mut Vec<OsString>)` | Pre-parse pass: `-5` → `-n 5`, stops at `--`, ignores `-n` / `-c` / `-n5` / `--lines=5` |
| `fn parse_num(&str) -> Result<u64, String>` | Decimal u64 parser — GNU error format on failure |
| `fn read_n_lines<R,W>(reader, writer, n)` | `BufRead::read_until(b'\n')` × N — raw-byte line slicing, preserves arbitrary bytes, handles unterminated final line |
| `fn read_n_bytes<R,W>(reader, writer, n)` | `BufReader::take(n)` + `io::copy` — byte-exact, may split multi-byte UTF-8 (D-48) |
| `pub fn uumain<I>(args) -> i32` | Operand loop: stdin fallback, `-` operand, multi-file header toggling, per-file error reporting (Pattern E), MSYS path conversion via `gow_core::path::try_convert_msys_path` |

14 unit tests cover: shorthand rewrite correctness (neg digits / `-n` / `-c` / `--` boundary / glued `-n5`), `read_n_lines` under/over/zero/unterminated, `read_n_bytes` exact/over/zero, `parse_num` valid/invalid cases.

### Task 2 — 18 integration tests in `crates/gow-head/tests/integration.rs` (commit `5e7b0c9`)

| Test | Covers |
|------|--------|
| `test_head_default_10` | Default N=10 line output on 15-line fixture |
| `test_head_n_3` | `-n 3` emits first 3 lines |
| `test_head_shorthand_5` | `-5` ≡ `-n 5` (D-05) |
| `test_head_n_attached_value` | `-n5` glued short-with-value form |
| `test_head_c_10` | `-c 10` of `"hello world\n"` → `"hello worl"` (byte-exact) |
| `test_head_c_over_size` | `-c 100` on 2-byte file returns 2 bytes, exit 0 |
| `test_head_c_mid_multibyte_splits_bytes` | `-c 2` on `"안"` (3-byte UTF-8) emits raw `[0xEC, 0x95]` (D-48) |
| `test_head_n_0_empty_output` | `-n 0` emits nothing |
| `test_head_multi_file_headers` | `==> file <==` appears in multi-file output |
| `test_head_q_suppresses_headers` | `-q` strips all headers |
| `test_head_v_forces_header_single_file` | `-v` adds header even for single file |
| `test_head_stdin_no_operand` | No operand → reads stdin |
| `test_head_dash_operand` | `-` operand → reads stdin |
| `test_head_nonexistent_file` | Missing file → exit 1, stderr starts `head:` |
| `test_head_partial_failure_continues` | `head missing ok.txt` emits ok content and exits 1 (Pattern E) |
| `test_head_utf8_preserved` | Korean `"안녕\n"` round-trips byte-exact through `-n 1` |
| `test_head_empty_file` | Empty file → exit 0, empty stdout |
| `test_head_bad_flag_exits_1` | Unknown flag → exit 1 (D-02) |

## Verification Results

```
cargo test -p gow-head            -> 14 unit + 18 integration = 32 passed / 0 failed
cargo clippy -p gow-head --all-targets -- -D warnings (in isolation) -> clean

Spot check (binary):
  printf '1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n' | ./target/.../debug/head.exe -5
  → emits 1..5, exit 0
```

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written. The plan's SECTION A-G contents produced a working implementation on first compile; integration tests passed on first run.

### Deferred (Out of Scope)

**1. [Scope Boundary] pre-existing unused import in `crates/gow-core/src/args.rs`**

- **Observed:** `+ use clap::error::ErrorKind;` present uncommitted in the worktree; causes `-D warnings` failure in any workspace-wide clippy invocation.
- **Why not fixed:** gow-core is Wave 0's responsibility (plan 03-01). Editing it from a Wave 1 plan would expand 03-03's blast radius beyond TEXT-01.
- **Logged to:** `.planning/phases/03-filesystem/deferred-items.md`
- **Workaround:** Stashed the change for the scoped clippy run; confirmed `gow-head` itself is clippy-clean; restored the stash. No change to gow-core committed from 03-03.

**2. [Worktree state observation] `.planning/STATE.md` uncommitted diff + empty `crates/gow-cat/tests/` scratch directory**

- Both are leftover artifacts from the parallel Wave 1 executor (plan 03-02) already committed ahead of 03-03 in this worktree (commit `68603fd`). Per the parallel_execution directive, 03-03 did not touch STATE.md.

### Authentication Gates

None.

## Commits

| Commit | Message |
|--------|---------|
| `d90bc87` | feat(03-03): implement gow-head uumain with -n/-c/-q/-v + numeric shorthand |
| `5e7b0c9` | test(03-03): add 18 integration tests for gow-head |

## Success Criteria (from plan)

- [x] -n, -c, -q, -v flags implemented
- [x] Numeric shorthand (-5 = -n 5) via pre-parse step
- [x] Multi-file `==> file <==` headers with blank-line separator
- [x] Per-file error reporting (Pattern E)
- [x] UTF-8 byte preservation (verified via Korean fixture test)
- [x] Integration tests >= 15 (delivered 18)
- [x] Clippy clean (gow-head in isolation — pre-existing gow-core issue logged as deferred)

## Self-Check: PASSED

**Files created verified:**
- `crates/gow-head/tests/integration.rs` — FOUND (256 lines)
- `.planning/phases/03-filesystem/deferred-items.md` — FOUND

**Files modified verified:**
- `crates/gow-head/src/lib.rs` — modified from 22 lines (stub) to 388 lines (real); confirmed via `wc -l`

**Commits verified present:**
- `d90bc87` — confirmed via `git log --oneline -5`
- `5e7b0c9` — confirmed via `git log --oneline -5`

**Functional verification:**
- `cargo test -p gow-head` → 14 unit + 18 integration = 32 passed, 0 failed
- `cargo clippy -p gow-head --all-targets -- -D warnings` (scoped) → clean
- `head.exe -5 <piped 10 lines>` → emits first 5 lines, exit 0
