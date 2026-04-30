---
phase: 10-new-utilities-wave1
plan: "04"
subsystem: gow-od
tags: [od, octal-dump, hex-dump, type-specifier, address-format, tdd]
dependency_graph:
  requires:
    - "10-01 (gow-od crate scaffold)"
  provides:
    - "crates/gow-od/src/lib.rs (full od implementation)"
    - "crates/gow-od/tests/integration.rs (19 integration tests)"
  affects:
    - "crates/gow-od/src/lib.rs (replaced stub)"
tech_stack:
  added: []
  patterns:
    - "AddrFmt enum for -A address format dispatch (o/x/d/n)"
    - "TypeSpec enum for -t type specifier dispatch (o/x/d/u/c + sizes 1/2/4/8)"
    - "u16::from_le_bytes / u32::from_le_bytes for little-endian word assembly"
    - "3-char field width for -t c (named escapes + printable chars)"
    - "16 bytes/row GNU default; partial last row with zero-padded unit assembly"
key_files:
  created: []
  modified:
    - crates/gow-od/src/lib.rs
    - crates/gow-od/tests/integration.rs
decisions:
  - "format_char uses 3-char fields (not 4) — the 1-space separator creates 4-char visible columns; confirmed via test-driven comparison with GNU od output spec"
  - "Partial last unit (fewer bytes than unit_size) padded with zeros to complete word — consistent with GNU od behavior for odd-sized inputs"
  - "Reading all input into memory acceptable for Phase 10 scope; documented as limitation vs streaming"
  - "TDD RED/GREEN: 17 tests failed with stub, all 19 green after implementation; 1 auto-fix for format_char field width"
metrics:
  duration: "5 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 1
  tasks_total: 1
  files_created: 0
  files_modified: 2
---

# Phase 10 Plan 04: gow-od Implementation Summary

**One-liner:** Full od octal/hex dump implementation with AddrFmt/TypeSpec enums, little-endian word reading, GNU-compatible 16-bytes-per-row output, all four address formats (-A o/x/d/n), five type specifiers (-t o/x/d/u/c) with sizes 1-8, and -N byte limit.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 (RED) | Add failing integration tests for all od behaviors | 2622fec | Done |
| 1 (GREEN) | Implement full gow-od with AddrFmt, TypeSpec, dump engine | 9038e2a | Done |

## Verification Results

- `cargo test -p gow-od` exits 0 — 14 unit tests + 19 integration tests all pass
- `grep "AddrFmt" crates/gow-od/src/lib.rs` — FOUND (line 58)
- `grep "TypeSpec" crates/gow-od/src/lib.rs` — FOUND (line 89)
- `grep "u16::from_le_bytes" crates/gow-od/src/lib.rs` — FOUND (line 135)
- `grep 'default_value = "o2"' crates/gow-od/src/lib.rs` — FOUND (line 42)
- `grep "not implemented" crates/gow-od/src/lib.rs` — NO MATCH (stub removed)
- `echo -n "" | cargo run -p gow-od` — outputs `0000000\n`
- `echo -n "ab" | cargo run -p gow-od` — outputs `0000000 061141\n0000002\n`
- `cargo build --workspace` — exits 0, no regressions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] format_char field width correction**
- **Found during:** Task 1 (GREEN phase) — tests `od_t_c_chars` and `od_t_c_control_chars` failed
- **Issue:** Initial `format_char` implementation used 4-char fields (`"   A"`, `"  \\n"`). But GNU od `-t c` uses 3-char value fields — the leading space separator creates 4 visible columns. With 4-char fields plus the separator, output was 1 char too wide per value.
- **Fix:** Changed `format_char` to return 3-char strings: printable → `"{:>3}"`, named escapes → `" \\n"` etc. (1 leading space + 2-char escape), non-printable → 3-digit octal.
- **Files modified:** `crates/gow-od/src/lib.rs` (format_char function + unit tests)
- **Commit:** 9038e2a (included in GREEN commit)

## TDD Gate Compliance

- RED gate commit: `2622fec` (test(10-04): add failing integration tests for gow-od)
- GREEN gate commit: `9038e2a` (feat(10-04): implement gow-od octal/hex dump utility)
- REFACTOR: Not needed — implementation is clean

## Known Stubs

None. All od features specified in the plan are implemented and tested.

## Threat Surface Scan

No new threat surface introduced beyond what's documented in the plan's `<threat_model>`. The `parse_type_spec` function rejects unknown type strings with an explicit error (T-10-04-02 mitigation applied). Large input reads are bounded by `-N` limit (T-10-04-01 accepted).

## Self-Check: PASSED

- `crates/gow-od/src/lib.rs` — FOUND (498 lines, > 200 minimum)
- `crates/gow-od/tests/integration.rs` — FOUND (272 lines, > 80 minimum, contains `od_default_octal_two_byte`)
- RED commit 2622fec — FOUND (`git log --oneline | grep 2622fec`)
- GREEN commit 9038e2a — FOUND (`git log --oneline | grep 9038e2a`)
- lib.rs exports `uumain` — FOUND
- lib.rs contains `AddrFmt`, `TypeSpec`, `u16::from_le_bytes`, `default_value = "o2"` — all FOUND
- `cargo test -p gow-od` — 33 tests total (14 unit + 19 integration), all PASSED
