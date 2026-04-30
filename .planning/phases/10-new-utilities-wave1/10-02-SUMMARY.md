---
phase: 10-new-utilities-wave1
plan: "02"
subsystem: stream-utilities
tags: [seq, sleep, tac, fold, tdd, scaled-integer, stream-processing]
dependency_graph:
  requires:
    - 10-01 (crate scaffolds must exist)
  provides:
    - crates/gow-seq (full implementation)
    - crates/gow-sleep (full implementation)
    - crates/gow-tac (full implementation)
    - crates/gow-fold (full implementation)
  affects:
    - crates/gow-seq/src/lib.rs
    - crates/gow-seq/tests/integration.rs
    - crates/gow-sleep/src/lib.rs
    - crates/gow-sleep/tests/integration.rs
    - crates/gow-tac/src/lib.rs
    - crates/gow-tac/tests/integration.rs
    - crates/gow-fold/src/lib.rs
    - crates/gow-fold/tests/integration.rs
tech_stack:
  added: []
  patterns:
    - scaled-integer arithmetic for seq (decimal_places() + 10_i64.pow + i64 accumulator)
    - allow_hyphen_values=true on positional Vec for negative-number arguments
    - Duration::from_secs_f64 for fractional sleep
    - byte-slice split-on-newline + Vec::reverse for tac
    - BufRead::lines + sliding-window space-search for fold -s word-boundary
    - extended window search [pos..chunk_end+1] to handle boundary-aligned spaces
key_files:
  created: []
  modified:
    - crates/gow-seq/src/lib.rs
    - crates/gow-seq/tests/integration.rs
    - crates/gow-sleep/src/lib.rs
    - crates/gow-sleep/tests/integration.rs
    - crates/gow-tac/src/lib.rs
    - crates/gow-tac/tests/integration.rs
    - crates/gow-fold/src/lib.rs
    - crates/gow-fold/tests/integration.rs
decisions:
  - "seq uses allow_hyphen_values=true on the numbers Vec so negative increments like -1 are accepted as positional values, not flags"
  - "fold -s word-boundary search window is [pos..chunk_end+1] (inclusive of chunk_end) to handle the common case where the break space is exactly at the width boundary (e.g. 'hello world' at width=11)"
  - "tac reads entire file/stdin into memory (Vec<u8>) — acceptable per RESEARCH anti-patterns; streaming reverse not implemented in Phase 10"
  - "sleep sums multiple duration arguments (GNU-compatible: sleep 1 2 3 = 6 seconds total)"
  - "seq rejects NaN/Infinity via is_finite() check (T-10-02-03 threat model mitigation)"
metrics:
  duration: "8 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 8
---

# Phase 10 Plan 02: seq, sleep, tac, fold Implementation Summary

**One-liner:** Four GNU stream utilities fully implemented — seq with scaled-integer decimal precision (no f64 accumulation), sleep with fractional-second Duration::from_secs_f64, tac with byte-slice reversal, and fold with byte-width wrapping and -s word-boundary search.

## Tasks Completed

| Task | Description | Commit (RED) | Commit (GREEN) | Status |
|------|-------------|-------------|----------------|--------|
| 1 | Implement gow-seq + gow-sleep | d2ba565 | 927193c | Done |
| 2 | Implement gow-tac + gow-fold | 0819dd2 | 00e9965 | Done |

## Verification Results

- `cargo test -p gow-seq` exits 0 — 9 tests passing
- `cargo test -p gow-sleep` exits 0 — 4 tests passing
- `cargo test -p gow-tac` exits 0 — 5 tests passing
- `cargo test -p gow-fold` exits 0 — 8 tests passing
- `cargo build --workspace` exits 0 — no regressions
- `cargo run -p gow-seq -- 0.1 0.1 1.0` produces exactly 10 lines, last line "1.0"
- `cargo run -p gow-sleep -- 0` exits 0
- `echo "abcdefghij" | cargo run -p gow-fold -- -w 3` produces "abc\ndef\nghi\nj\n"
- `printf "a\nb\nc\n" | cargo run -p gow-tac` produces "c\nb\na\n"
- `grep "decimal_places" crates/gow-seq/src/lib.rs` finds match
- `grep "10_i64.pow" crates/gow-seq/src/lib.rs` finds match
- `grep "Duration::from_secs_f64" crates/gow-sleep/src/lib.rs` finds match
- No stub "not implemented" messages remain in any of the 4 lib.rs files

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clap rejects negative increments as flags**
- **Found during:** Task 1 GREEN phase (seq_negative_increment test failure)
- **Issue:** `seq 5 -1 1` caused clap to interpret `-1` as an unknown flag, exiting with error about "unexpected argument '-1'"
- **Fix:** Added `#[arg(allow_hyphen_values = true)]` to the `numbers: Vec<String>` positional argument in Cli, allowing hyphen-prefixed values to be treated as positional strings
- **Files modified:** `crates/gow-seq/src/lib.rs`
- **Commit:** 927193c (included in GREEN commit)

**2. [Rule 1 - Bug] fold -s word-boundary search window too narrow**
- **Found during:** Task 2 GREEN phase (fold_word_boundary_with_s test failure)
- **Issue:** Standard window `[pos..chunk_end]` found space at index 5 in "hello world" (breaking at "hello ") instead of index 11 (breaking at "hello world"). The GNU-expected behavior breaks just before the space that follows the width boundary.
- **Fix:** Extended search window to `[pos..chunk_end+1].min(line.len())` so boundary-aligned spaces are discovered; break is placed before the space (outputting `line[pos..space_pos]`)
- **Files modified:** `crates/gow-fold/src/lib.rs`
- **Commit:** 00e9965 (included in GREEN commit)

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED (seq+sleep tests) | d2ba565 | test(10-02): add failing integration tests for seq and sleep (RED) |
| GREEN (seq+sleep impl) | 927193c | feat(10-02): implement gow-seq and gow-sleep (GREEN) |
| RED (tac+fold tests) | 0819dd2 | test(10-02): add failing integration tests for tac and fold (RED) |
| GREEN (tac+fold impl) | 00e9965 | feat(10-02): implement gow-tac and gow-fold (GREEN) |

Both RED gates: tests failed on stubs as expected. Both GREEN gates: all tests passed after implementation.

## Known Stubs

None. All four utilities have full implementations. Stub "not implemented" messages have been replaced.

**Phase 10 limitations documented (not stubs):**
- `fold` counts bytes only (no Unicode width-aware column counting) — documented in code
- `fold -s` with no space in window falls back to hard break (GNU-compatible)
- `sleep` does not support `s/m/h/d` suffix notation — only plain decimal seconds
- `tac` reads entire input into memory (OOM risk on very large files) — documented per RESEARCH.md

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns beyond what the plan's threat model already covers. The three trust boundaries from the plan's `<threat_model>` were all addressed:

| Threat ID | Status |
|-----------|--------|
| T-10-02-01 | Accepted — tac reads entire file into memory; documented limitation |
| T-10-02-02 | Accepted — sleep with very large argument blocks; GNU behavior |
| T-10-02-03 | Mitigated — seq rejects NaN/Infinity via `is_finite()` check before scaling; zero increment rejected with explicit error |

## Self-Check: PASSED

- `crates/gow-seq/src/lib.rs` — FOUND
- `crates/gow-seq/tests/integration.rs` — FOUND
- `crates/gow-sleep/src/lib.rs` — FOUND
- `crates/gow-sleep/tests/integration.rs` — FOUND
- `crates/gow-tac/src/lib.rs` — FOUND
- `crates/gow-tac/tests/integration.rs` — FOUND
- `crates/gow-fold/src/lib.rs` — FOUND
- `crates/gow-fold/tests/integration.rs` — FOUND
- RED seq+sleep commit d2ba565 — FOUND
- GREEN seq+sleep commit 927193c — FOUND
- RED tac+fold commit 0819dd2 — FOUND
- GREEN tac+fold commit 00e9965 — FOUND
