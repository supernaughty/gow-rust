---
phase: 11-new-utilities-wave2
plan: "05"
subsystem: gow-test
tags: [test, posix, condition-evaluator, bracket-mode, tdd]
dependency_graph:
  requires: ["11-01"]
  provides: ["gow-test crate with POSIX test evaluator and bracket mode"]
  affects: ["crates/gow-test"]
tech_stack:
  added: []
  patterns:
    - "Raw argv parsing without clap (test is not a flag-parsing utility)"
    - "Recursive descent parser for POSIX test expression grammar"
    - "--_bracket_ sentinel mechanism for [ alias detection from [.bat shim"
    - "std::fs::metadata for file predicate evaluation"
    - "gow_core::path::try_convert_msys_path for MSYS/Unix path conversion"
key_files:
  created: []
  modified:
    - crates/gow-test/src/lib.rs
    - crates/gow-test/tests/integration.rs
decisions:
  - "Used recursive descent parser with -o < -a < ! < primary precedence for correct POSIX semantics"
  - "Empty test (no args) exits 1 not 2 — POSIX specifies false, not error"
  - "--_bracket_ sentinel approach: bracket mode enforces trailing ] presence, returns exit 2 if missing"
  - "Windows -x predicate: file exists with .exe/.bat/.com/.cmd extension (no POSIX executable bit)"
  - "Windows -r: file readable if accessible (no POSIX read bit semantics)"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-29T21:05:24Z"
  tasks_completed: 1
  files_modified: 2
  tdd_commits: 2
---

# Phase 11 Plan 05: gow-test POSIX Condition Evaluator Summary

POSIX test evaluator with full predicate set, recursive descent parser, and --_bracket_ sentinel mechanism for [ alias detection; all 18 integration tests green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| RED | Add failing integration tests for gow-test | 2a69205 | crates/gow-test/tests/integration.rs |
| GREEN | Implement gow-test POSIX evaluator | 4100de1 | crates/gow-test/src/lib.rs |

## Implementation Details

### Operator Coverage

**File predicates:** `-f` (regular file), `-d` (directory), `-e` (exists, any type), `-r` (readable), `-w` (writable), `-x` (executable, Windows: .exe/.bat/.com/.cmd), `-s` (non-zero size), `-L` (symlink)

**String predicates:** `-z STRING` (zero length → true), `-n STRING` (non-zero → true), `STRING1 = STRING2`, `STRING1 != STRING2`, `STRING1 < STRING2`, `STRING1 > STRING2`

**Integer predicates:** `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge` (all i64 comparisons)

**Boolean operators:** `! EXPR` (negation, right-associative), `EXPR1 -a EXPR2` (AND), `EXPR1 -o EXPR2` (OR), `( EXPR )` (grouping)

### Parser Architecture

Recursive descent with operator precedence (lowest to highest):
- `parse_expr` handles `-o` (OR)
- `parse_and_expr` handles `-a` (AND)
- `parse_not_expr` handles `!` (NOT, right-associative)
- `parse_primary` handles file/string/integer predicates, parentheses, bare strings

### Bracket Mode

The `--_bracket_` sentinel is injected by `extras/bin/[.bat` when invoking `test.exe` as `[`:
- Sentinel detected at `args_vec[1]` position
- Sentinel stripped from expression arguments
- Trailing `]` required and stripped; exit 2 if missing

### Exit Code Semantics

Per POSIX (opposite of `expr`):
- `0` = condition is TRUE
- `1` = condition is FALSE (including empty expression)
- `2` = usage/syntax error

## Deviations from Plan

None — plan executed exactly as written. TDD protocol followed: RED commit (2a69205) → GREEN commit (4100de1).

## TDD Gate Compliance

- RED gate: commit `2a69205` — failing tests written before implementation
- GREEN gate: commit `4100de1` — implementation makes all 18 tests pass
- REFACTOR gate: not needed (code was clean as written)

## Verification Results

```
cargo test -p gow-test: 18 passed; 0 failed
cargo build --workspace: Finished (no errors)

Smoke tests:
  test -z ""        → exit 0 (true)
  test -n "hello"   → exit 0 (true)
  test 5 -gt 3      → exit 0 (true)
  test 3 -gt 5      → exit 1 (false)
  test (no args)    → exit 1 (false, NOT error)
  test --_bracket_ -z "" ]  → exit 0 (bracket mode)
  test --_bracket_ -z ""    → exit 2 (missing ])
  test ! -f /nonexistent    → exit 0 (negation of false = true)
```

## Self-Check: PASSED

- crates/gow-test/src/lib.rs: FOUND (221 lines, exports `uumain` and `evaluate_test`)
- crates/gow-test/tests/integration.rs: FOUND (190 lines, 18 tests)
- Commit 2a69205: FOUND (RED — test)
- Commit 4100de1: FOUND (GREEN — feat)
- `grep "not implemented" crates/gow-test/src/lib.rs`: no match (PASS)
- `grep -c "_bracket_"`: 4 matches (PASS, >= 2 required)
- `grep -c "metadata|symlink_metadata"`: 8 matches (PASS, >= 2 required)
- `grep -c "-z|-n|-eq|-gt"`: 9 matches (PASS, >= 4 required)
