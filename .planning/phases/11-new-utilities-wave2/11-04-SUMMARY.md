---
phase: 11-new-utilities-wave2
plan: "04"
subsystem: format-expression-utilities
tags: [printf, expr, format-string, recursive-descent, exit-codes, tdd, wave2]
dependency_graph:
  requires:
    - 11-01 (workspace scaffold — gow-printf/gow-expr stubs)
  provides:
    - crates/gow-printf/src/lib.rs (full printf implementation)
    - crates/gow-expr/src/lib.rs (full expr implementation)
  affects:
    - crates/gow-printf/tests/integration.rs
    - crates/gow-expr/tests/integration.rs
tech_stack:
  added: []
  patterns:
    - Manual argv parsing (no clap) — format string + positional args
    - Extra-args-repeat loop (printf format string reused for batches of args)
    - Recursive-descent parser with depth limit (expr, 7 levels)
    - Inverted exit code semantics (expr: 0=non-null, 1=null, 2=syntax-error)
    - regex crate for linear-time colon operator (T-11-04-03)
    - TDD RED/GREEN pattern — tests committed before implementation
key_files:
  created: []
  modified:
    - crates/gow-printf/src/lib.rs
    - crates/gow-printf/tests/integration.rs
    - crates/gow-expr/src/lib.rs
    - crates/gow-expr/tests/integration.rs
decisions:
  - "printf uses Peekable<Chars> for format string scanning to correctly handle multi-digit octal/hex escapes"
  - "expr recursion depth limit is 100 (T-11-04-01); depth parameter threaded through all 7 parse_* functions"
  - "expr colon operator uses regex crate (linear-time) — no catastrophic backtracking possible (T-11-04-03)"
  - "expr integer arithmetic uses checked_add/checked_sub/checked_mul/checked_div/checked_rem to avoid i64 overflow panic"
  - "printf %05.2f: zero-pad and precision handled separately to produce '03.10' not '3.10' or ' 03.10'"
metrics:
  duration_seconds: 236
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 4
---

# Phase 11 Plan 04: printf and expr Implementation

**One-liner:** C-style format string evaluator (printf) with extra-args repeat loop and recursive-descent arithmetic/string expression evaluator (expr) with inverted exit code semantics (0=non-null, 1=null/zero, 2=syntax-error).

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 (RED) | Add failing integration tests for gow-printf | 5d49554 | crates/gow-printf/tests/integration.rs |
| 1 (GREEN) | Implement gow-printf — format string evaluator | 5af63cb | crates/gow-printf/src/lib.rs |
| 2 (RED) | Add failing integration tests for gow-expr | 427e3d4 | crates/gow-expr/tests/integration.rs |
| 2 (GREEN) | Implement gow-expr — recursive-descent evaluator | 5e3e4d1 | crates/gow-expr/src/lib.rs |

## Verification Results

- `cargo test -p gow-printf --test integration`: 9/9 PASS
- `cargo test -p gow-expr --test integration`: 13/13 PASS
- `cargo build --workspace`: PASS (no regressions)
- `expr 3 + 4` → "7", exit 0: PASS
- `expr 3 - 3` → "0", exit 1 (NOT 0): PASS (critical exit code semantics)
- `expr` (no args) → exit 2: PASS
- `printf "%d\n" 1 2 3` → "1\n2\n3\n" (format repeat): PASS
- `printf "%05.2f\n" 3.1` → "03.10": PASS

## TDD Gate Compliance

Both tasks followed strict RED/GREEN discipline:

- Task 1 RED: `test(11-04): add failing integration tests for gow-printf` (5d49554) — 8/9 tests failed on stub
- Task 1 GREEN: `feat(11-04): implement gow-printf — format string evaluator` (5af63cb) — 9/9 pass
- Task 2 RED: `test(11-04): add failing integration tests for gow-expr` (427e3d4) — 13/13 tests failed on stub
- Task 2 GREEN: `feat(11-04): implement gow-expr — recursive-descent expression evaluator` (5e3e4d1) — 13/13 pass

## Deviations from Plan

None — plan executed exactly as written.

## Threat Model Coverage

| Threat ID | Status | Implementation |
|-----------|--------|---------------|
| T-11-04-01 | Mitigated | Recursion depth counter (MAX_DEPTH=100) threaded through all 7 parse_* functions |
| T-11-04-02 | Mitigated | Rust's memory-safe string handling; unknown % specs emit verbatim; no C sprintf |
| T-11-04-03 | Mitigated | `regex` crate used for colon operator — linear-time guarantee, no catastrophic backtracking |

## Known Stubs

None — both implementations are complete.

## Self-Check: PASSED

- [x] crates/gow-printf/src/lib.rs exists (404 lines >= 120 min)
- [x] crates/gow-printf/tests/integration.rs exists (76 lines >= 60 min)
- [x] crates/gow-expr/src/lib.rs exists (304 lines >= 200 min)
- [x] crates/gow-expr/tests/integration.rs exists (93 lines >= 70 min)
- [x] Commits 5d49554, 5af63cb, 427e3d4, 5e3e4d1 present in git log
- [x] printf_format_repeats present in test file
- [x] expr_zero_result_exits_1 present in test file
- [x] uumain exported from both lib.rs files
- [x] format_one_pass / format_spec present in printf lib
- [x] parse_or / parse_and / parse_comparison / parse_additive / parse_multiplicative present in expr lib
- [x] is_null() present in expr lib (null-check for exit code)
- [x] cargo build --workspace exits 0
