---
phase: 04-s04
plan: "08"
subsystem: sort
tags: [rust, sort, key-field, gnu-sort, gow-sort]

# Dependency graph
requires:
  - phase: 04-01
    provides: gow-sort crate with external merge sort, SortConfig, compare_lines
provides:
  - KeySpec struct for -k KEYDEF parsing
  - extract_key_field function for field-based key extraction
  - -k/--key and -t/--field-separator CLI arguments
  - sort -k N field-based sorting (whitespace and custom separator)
  - 4 integration tests covering key field sorting scenarios
affects: [04-09, 05-s05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Key spec parsing: modifier letters stripped from KEYDEF before numeric field parsing"
    - "Field extraction: split on whitespace runs (GNU default) or custom separator byte"
    - "Multi-key comparison: iterate keys, fall back to whole-line on all-equal"

key-files:
  created: []
  modified:
    - crates/gow-sort/src/lib.rs
    - crates/gow-sort/tests/integration.rs

key-decisions:
  - "SortConfig Copy removed to allow Vec<KeySpec>; write_sorted and merge_temp_files take &SortConfig"
  - "compare_bytes split from compare_lines as the low-level comparison primitive"
  - "merge_temp_files captures keys/separator/numeric/ignore_case/reverse by value into the kmerge_by closure (SortConfig not Send)"
  - "parse_single_key: malformed KEYDEF silently falls back to field 1 via unwrap_or(1) — matches T-04-08-01 accept disposition"

patterns-established:
  - "KeySpec: start_field/end_field (1-based), numeric/reverse/ignore_case modifier flags"
  - "extract_key_field: bounds-clamped with .min(n-1) to prevent panics on short lines (T-04-08-02 mitigation)"

requirements-completed: ["R008"]

# Metrics
duration: 15min
completed: 2026-04-25
---

# Phase 04 Plan 08: sort -k Key Field Support Summary

**GNU sort -k KEYDEF with field extraction, custom separator (-t), numeric/reverse/ignore-case key modifiers, and 12 integration tests passing**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-25
- **Completed:** 2026-04-25
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `KeySpec` struct and `parse_key_specs`/`parse_single_key` parsers for -k KEYDEF format
- Implemented `extract_key_field` with bounds-clamped field indexing for both whitespace and custom byte separators
- Refactored `compare_lines` to dispatch through key specs with per-key numeric/reverse/ignore_case modifiers, then fall back to `compare_bytes`
- Extended `SortConfig` with `keys: Vec<KeySpec>` and `field_separator: Option<u8>`; removed `Copy`, updated all call sites to pass `&SortConfig`
- Registered `-k`/`--key` (Append action) and `-t`/`--field-separator` args in `uu_app()`
- 4 new integration tests covering: lex key sort, numeric key, reverse key, colon-separator key — all 12 tests pass

## Task Commits

1. **Task 1: Add KeySpec, extract_key_field, -k/-t args** - `2671450` (feat)
2. **Task 2: Integration tests for -k key field sorting** - `93e849d` (test)

## Files Created/Modified

- `crates/gow-sort/src/lib.rs` - KeySpec, extract_key_field, updated SortConfig, compare_lines refactor, -k/-t uu_app args
- `crates/gow-sort/tests/integration.rs` - 4 new -k integration tests appended

## Decisions Made

- SortConfig `Copy` removed because `Vec<KeySpec>` is not `Copy`; `write_sorted` and `merge_temp_files` signatures changed to `&SortConfig`
- `merge_temp_files` captures individual config fields by value into the `kmerge_by` closure to avoid lifetime issues with `&SortConfig`
- `compare_bytes` extracted as a separate function; `compare_lines` delegates to it after key extraction
- Malformed KEYDEF (unparseable field number) falls back to field 1 silently — consistent with T-04-08-01 accept disposition

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - build succeeded on first attempt, all 12 tests passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- sort -k KEYDEF fully operational; closes VERIFICATION.md truths #29 and #30
- All 12 sort integration tests passing
- Ready for plan 04-09

## Self-Check: PASSED

- `crates/gow-sort/src/lib.rs` exists and compiles
- `crates/gow-sort/tests/integration.rs` contains all 4 new test functions
- Commit `2671450` (Task 1) exists
- Commit `93e849d` (Task 2) exists
- `cargo test -p gow-sort` exits 0 (12/12 tests pass)

---
*Phase: 04-s04*
*Completed: 2026-04-25*
