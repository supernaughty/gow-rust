---
phase: 11-new-utilities-wave2
plan: "03"
subsystem: new-utilities
tags: [join, split, tdd, merge-join, file-splitter, alphabetic-suffix, wave2]
dependency_graph:
  requires:
    - 11-01 (gow-join stub, gow-split stub)
  provides:
    - crates/gow-join/src/lib.rs (full merge-join implementation)
    - crates/gow-split/src/lib.rs (full file-splitter implementation)
    - crates/gow-join/tests/integration.rs (5 integration tests)
    - crates/gow-split/tests/integration.rs (6 integration tests)
  affects:
    - cargo test -p gow-join -p gow-split (now green)
tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN cycle with integration tests
    - merge-join loop with Option<String> line state per reader
    - next_suffix alphabetic cycling (aa→ab→...→zz→aaa)
    - parse_bytes with K/M/G suffix multipliers
    - get_field/other_fields 1-based field extraction
key_files:
  created:
    - crates/gow-join/tests/integration.rs
    - crates/gow-split/tests/integration.rs
  modified:
    - crates/gow-join/src/lib.rs
    - crates/gow-split/src/lib.rs
decisions:
  - "join: drain-remaining-file pattern with Option<String> state avoids borrow conflicts in merge loop"
  - "join: sort-order warning emitted to stderr when current key < previous key (T-11-03-03 mitigation)"
  - "split: -l mode iterates byte buffer line-by-line instead of reading again; avoids double allocation"
  - "split: next_suffix extends vec when all-z (zz→aaa) exactly as spec'd in RESEARCH Pattern 6"
  - "split -n: ceiling division (total+chunks-1)/chunks distributes bytes evenly"
metrics:
  duration_seconds: 450
  completed_date: "2026-04-30"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 2
---

# Phase 11 Plan 03: gow-join and gow-split Implementation Summary

**One-liner:** Merge-join implementation for gow-join (sorted-file join on configurable key fields with -1/-2/-t/-a/-v support) and file splitter for gow-split (-l lines/-b bytes/-n chunks with alphabetic suffix cycling aa→zz→aaa).

## Tasks Completed

| # | Name | Commit (RED) | Commit (GREEN) | Files |
|---|------|--------------|----------------|-------|
| 1 | Implement gow-split (file splitter with alphabetic suffix generation) | 55db80e | 4b22746 | crates/gow-split/src/lib.rs, tests/integration.rs |
| 2 | Implement gow-join (sorted-file merge join on key field) | 35b78b5 | a94fa80 | crates/gow-join/src/lib.rs, tests/integration.rs |

## TDD Gate Compliance

- Task 1 (gow-split): RED commit `55db80e` (6 tests, 5 failed on stub) → GREEN commit `4b22746` (6/6 pass)
- Task 2 (gow-join): RED commit `35b78b5` (5 tests, 4 failed on stub) → GREEN commit `a94fa80` (5/5 pass)

## Verification Results

- `cargo test -p gow-split`: 6/6 passed
- `cargo test -p gow-join`: 5/5 passed
- `cargo build --workspace`: exits 0, no regressions
- `grep "not implemented" crates/gow-split/src/lib.rs`: 0 matches (stub removed)
- `grep "not implemented" crates/gow-join/src/lib.rs`: 0 matches (stub removed)
- `grep "next_suffix" crates/gow-split/src/lib.rs`: 2 matches (definition + call)
- `grep "parse_bytes" crates/gow-split/src/lib.rs`: 2 matches (definition + call)
- `grep "1000" crates/gow-split/src/lib.rs`: 3 matches (default present)
- `grep "get_field|field1|field2" crates/gow-join/src/lib.rs`: 17 matches
- `grep "sorted order" crates/gow-join/src/lib.rs`: 2 matches (sort warning present)

## Deviations from Plan

None - plan executed exactly as written. All algorithms implemented from RESEARCH.md Pattern 6 spec.

## Known Stubs

None. Both gow-join and gow-split are fully implemented.

## Threat Flags

No new threat surface introduced beyond what is documented in the plan's threat model:
- T-11-03-01 (split reads all input into memory) — accepted, documented
- T-11-03-02 (split prefix path is user-controlled) — accepted, consistent with GNU
- T-11-03-03 (join unsorted input warning) — mitigated: "file N is not in sorted order" warning emitted to stderr

## Self-Check: PASSED

- [x] crates/gow-split/src/lib.rs exists and contains next_suffix, parse_bytes, 1000
- [x] crates/gow-join/src/lib.rs exists and contains get_field, sorted order warning
- [x] crates/gow-split/tests/integration.rs contains split_by_lines
- [x] crates/gow-join/tests/integration.rs contains join_basic
- [x] Commits 55db80e, 4b22746, 35b78b5, a94fa80 present in git log
- [x] cargo test -p gow-split -p gow-join: 11/11 passed
- [x] cargo build --workspace: exits 0
