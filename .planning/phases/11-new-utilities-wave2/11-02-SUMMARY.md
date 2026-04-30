---
phase: 11-new-utilities-wave2
plan: "02"
subsystem: utilities-implementation
tags: [unlink, fmt, paste, tdd, integration-tests, phase11]
dependency_graph:
  requires:
    - 11-01 (crate scaffolds for gow-unlink, gow-fmt, gow-paste)
  provides:
    - crates/gow-unlink/src/lib.rs (full implementation)
    - crates/gow-unlink/tests/integration.rs (4 tests)
    - crates/gow-fmt/src/lib.rs (full implementation)
    - crates/gow-fmt/tests/integration.rs (5 tests)
    - crates/gow-paste/src/lib.rs (full implementation)
    - crates/gow-paste/tests/integration.rs (6 tests)
  affects:
    - cargo test -p gow-unlink -p gow-fmt -p gow-paste (all green)
tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN cycle (separate test commit then implementation commit)
    - Raw argv parsing without clap (gow-unlink, exact-1-arg enforcement)
    - Paragraph-aware word-wrap with flush_paragraph free function (gow-fmt)
    - ColSource enum with Box<dyn BufRead> dynamic dispatch per column (gow-paste)
    - Shared stdin iterator via &mut dyn Iterator for multi-dash paste - -
key_files:
  created: []
  modified:
    - crates/gow-unlink/src/lib.rs
    - crates/gow-unlink/tests/integration.rs
    - crates/gow-fmt/src/lib.rs
    - crates/gow-fmt/tests/integration.rs
    - crates/gow-paste/src/lib.rs
    - crates/gow-paste/tests/integration.rs
decisions:
  - "unlink skips clap and parses raw argv — enforces exactly 1 operand with explicit exit 2 on wrong count"
  - "fmt uses flush_paragraph as a free function (not closure) to avoid borrow checker complexity with &mut Vec + &mut dyn Write"
  - "paste uses ColSource enum with Stdin/Buffered variants; shared stdin passed as &mut dyn Iterator to handle paste - - alternating reads naturally"
  - "paste avoids unsafe mem::zeroed — initial approach had UB warnings; refactored to pass stdin iter by &mut dyn Iterator reference instead"
metrics:
  duration_seconds: 602
  completed_date: "2026-04-30"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 6
---

# Phase 11 Plan 02: gow-unlink, gow-fmt, gow-paste — Full Implementation

**One-liner:** Replaced three Wave 0 stubs with full implementations: unlink (exact-1-arg fs::remove_file), fmt (paragraph-aware word-wrap at -w width with blank-line paragraph separators), and paste (multi-file column zipper with -d delimiter cycling and shared-stdin - - support).

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 (RED) | Failing tests for gow-unlink + gow-fmt | ddc22c9 | tests/integration.rs x2 |
| 1 (GREEN) | Implement gow-unlink and gow-fmt | 6028919 | lib.rs x2 |
| 2 (RED) | Failing tests for gow-paste | 8ea39fc | crates/gow-paste/tests/integration.rs |
| 2 (GREEN) | Implement gow-paste | 4dc65bd | crates/gow-paste/src/lib.rs |

## Verification Results

- `cargo test -p gow-unlink`: 4/4 tests PASS
- `cargo test -p gow-fmt`: 5/5 tests PASS
- `cargo test -p gow-paste`: 6/6 tests PASS
- `cargo build --workspace`: PASS (no regressions)
- `grep "not implemented" crates/gow-unlink/src/lib.rs`: NO MATCH (stub replaced)
- `grep "not implemented" crates/gow-fmt/src/lib.rs`: NO MATCH (stub replaced)
- `grep "not implemented" crates/gow-paste/src/lib.rs`: NO MATCH (stub replaced)
- `grep "remove_file" crates/gow-unlink/src/lib.rs`: 1 match
- `grep -E "(para|flush)" crates/gow-fmt/src/lib.rs`: multiple matches (flush_paragraph, para_words)
- `grep "75" crates/gow-fmt/src/lib.rs`: 2 matches (comment + default_value)
- `grep -E "Box.*dyn BufRead" crates/gow-paste/src/lib.rs`: 1 match
- `grep "delimiters" crates/gow-paste/src/lib.rs`: 19 matches

## TDD Gate Compliance

| Phase | Commit | Gate |
|-------|--------|------|
| RED (unlink+fmt) | ddc22c9 | `test(11-02): add failing tests` — all tests confirmed failing on stubs |
| GREEN (unlink+fmt) | 6028919 | `feat(11-02): implement` — all 9 tests pass |
| RED (paste) | 8ea39fc | `test(11-02): add failing tests for gow-paste` — 5/6 tests confirmed failing |
| GREEN (paste) | 4dc65bd | `feat(11-02): implement gow-paste` — all 6 tests pass |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unsafe mem::zeroed() in initial paste implementation**
- **Found during:** Task 2 GREEN phase, cargo build warnings
- **Issue:** Initial ColReader design used `unsafe { std::mem::zeroed() }` as a dummy stdin_lines reference for non-stdin paths. Compiler warned this is UB (references must be non-null).
- **Fix:** Refactored to `ColSource` enum with `Stdin` and `Buffered(Box<dyn BufRead>)` variants; passed stdin as `&mut dyn Iterator<Item = io::Result<String>>` reference. No unsafe code in final implementation.
- **Files modified:** crates/gow-paste/src/lib.rs
- **Commit:** 4dc65bd (within the GREEN phase commit)

## Known Stubs

None — all three crates have full implementations with passing integration tests.

## Threat Flags

None — the three utilities (unlink, fmt, paste) match the threat surface described in the plan's threat_model. No new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- [x] crates/gow-unlink/src/lib.rs exists and has `remove_file`
- [x] crates/gow-unlink/tests/integration.rs exists and contains `unlink_removes_file`
- [x] crates/gow-fmt/src/lib.rs exists and has `flush_paragraph` + width 75
- [x] crates/gow-fmt/tests/integration.rs exists and contains `fmt_wraps_at_width`
- [x] crates/gow-paste/src/lib.rs exists and has `Box<dyn BufRead>` + `delimiters`
- [x] crates/gow-paste/tests/integration.rs exists and contains `paste_two_files`
- [x] Commits ddc22c9, 6028919, 8ea39fc, 4dc65bd present in git log
- [x] cargo test -p gow-unlink -p gow-fmt -p gow-paste: 15/15 tests green
- [x] cargo build --workspace exits 0
