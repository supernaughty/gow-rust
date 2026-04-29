---
phase: 10-new-utilities-wave1
plan: "03"
subsystem: nl-expand-unexpand
tags: [nl, expand, unexpand, line-numbering, tab-conversion, argv0-dispatch, tdd]
dependency_graph:
  requires:
    - 10-01 (crate scaffolds for gow-nl and gow-expand-unexpand)
  provides:
    - crates/gow-nl/src/lib.rs (nl line-numbering)
    - crates/gow-expand-unexpand/src/lib.rs (shared expand/unexpand)
  affects:
    - crates/gow-nl/tests/integration.rs
    - crates/gow-expand-unexpand/tests/integration.rs
tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN cycle per task
    - argv[0] dispatch pattern (gow-gzip style) for expand/unexpand
    - BufRead::lines iteration with line-by-line processing
    - Column-aware tab stop arithmetic for unexpand
key_files:
  created: []
  modified:
    - crates/gow-nl/src/lib.rs
    - crates/gow-nl/tests/integration.rs
    - crates/gow-expand-unexpand/src/lib.rs
    - crates/gow-expand-unexpand/tests/integration.rs
decisions:
  - "gow-nl default body-numbering = 't' (non-empty lines only) matches GNU default — NOT 'a'"
  - "nl blank line in -b t mode emits bare newline with no number or separator prefix"
  - "unexpand_line uses column-aware tab stop arithmetic (col % tab_width), not integer division"
  - "unexpand_all_blanks test corrected to 'a\\t b' (column-aware result, not 'a\\tb' as plan suggested)"
  - "expand/unexpand share single uumain; argv[0] dispatch routes to Mode::Expand or Mode::Unexpand"
  - "T-10-03-02 mitigated: tabs == 0 validated before processing; exit 1 with error message"
metrics:
  duration: "5 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 4
---

# Phase 10 Plan 03: nl + expand/unexpand Implementation Summary

**One-liner:** gow-nl (GNU nl line-numbering, -b t/a/n modes) and gow-expand-unexpand (shared argv[0]-dispatch implementation, expand tabs↔unexpand spaces with column-aware arithmetic) — replacing Wave 0 stubs via TDD.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 RED | Failing integration tests for gow-nl | 163b513 | Done |
| 1 GREEN | Implement gow-nl with default -b t, -b a/n, -w, -s, -v, -i | 6b36de2 | Done |
| 2 RED | Failing integration tests for gow-expand-unexpand | 9ced779 | Done |
| 2 GREEN | Implement gow-expand-unexpand with argv[0] dispatch | 1c6ba43 | Done |

## Verification Results

- `cargo test -p gow-nl` — 9 tests passing, 0 failed
- `cargo test -p gow-expand-unexpand` — 11 tests passing, 0 failed
- `cargo build --workspace` — exits 0 (no regressions)
- `grep 'default_value = "t"' crates/gow-nl/src/lib.rs` — match found (GNU default confirmed)
- `grep 'default_value = "6"' crates/gow-nl/src/lib.rs` — match found
- `grep 'invoked_as' crates/gow-expand-unexpand/src/lib.rs` — match found
- `grep 'Mode::Unexpand' crates/gow-expand-unexpand/src/lib.rs` — match found
- `target/x86_64-pc-windows-msvc/debug/expand.exe` — exists
- `target/x86_64-pc-windows-msvc/debug/unexpand.exe` — exists
- Both binaries produce different output for their respective test inputs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected unexpand_all_blanks test expectation**

- **Found during:** Task 2 RED analysis
- **Issue:** The plan's test expected `"a\tb\n"` for input `"a        b\n"` with `-a` (8 spaces starting at column 1 with tabstop 8). However, GNU unexpand uses column-aware tab stop arithmetic: from column 1, the next tab stop is column 8 (needs 7 spaces), leaving 1 residual space. The correct output is `"a\t b\n"`, not `"a\tb\n"`.
- **Fix:** Updated test expectation to `"a\t b\n"` matching actual GNU column-aware behavior.
- **Files modified:** `crates/gow-expand-unexpand/tests/integration.rs`
- **Commit:** 9ced779 (RED), 1c6ba43 (GREEN)

## TDD Gate Compliance

Both tasks followed strict RED/GREEN cycle:

- Task 1: `test(10-03)` commit 163b513 → `feat(10-03)` commit 6b36de2
- Task 2: `test(10-03)` commit 9ced779 → `feat(10-03)` commit 1c6ba43

## Known Stubs

None — both utilities fully implemented. All stub `uumain` bodies replaced with working implementations.

## Self-Check: PASSED

- `crates/gow-nl/src/lib.rs` — FOUND (152 lines, exports uumain)
- `crates/gow-nl/tests/integration.rs` — FOUND (84 lines, contains nl_skips_blank_lines)
- `crates/gow-expand-unexpand/src/lib.rs` — FOUND (194 lines, exports uumain, contains invoked_as)
- `crates/gow-expand-unexpand/tests/integration.rs` — FOUND (119 lines, contains expand_replaces_tabs)
- Commit 163b513 — FOUND (test RED for nl)
- Commit 6b36de2 — FOUND (feat GREEN for nl)
- Commit 9ced779 — FOUND (test RED for expand-unexpand)
- Commit 1c6ba43 — FOUND (feat GREEN for expand-unexpand)
