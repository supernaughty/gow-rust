---
phase: 08-code-review-fixes
plan: "02"
subsystem: compression
tags: [xz, liblzma, multi-stream, decompression, tdd]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    provides: gow-xz implementation that this plan fixes
provides:
  - "WR-04 fix: XzDecoder::new_multi_decoder replaces single-stream XzDecoder::new in gow-xz"
  - "Integration test proving multi-stream xz decompression works end-to-end"
affects: [08-code-review-fixes]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN: failing test committed before fix, fix committed after all tests pass"
    - "Inline fixture construction: concatenated xz streams built via liblzma::write::XzEncoder in test code (no binary fixtures)"

key-files:
  created: []
  modified:
    - crates/gow-xz/src/lib.rs
    - crates/gow-xz/tests/xz_tests.rs

key-decisions:
  - "Single-line fix only — no other changes to gow-xz/src/lib.rs (CONTEXT.md D-06: only WR-04 in scope)"
  - "Inline fixture construction via XzEncoder::new() twice into one buffer, consistent with D-02 (no binary fixtures in git)"

patterns-established:
  - "Multi-stream xz test pattern: two XzEncoder::finish() calls into same Vec<u8>, write to temp .xz file, verify both streams decoded"

requirements-completed: [FIX-04]

# Metrics
duration: 8min
completed: 2026-04-29
---

# Phase 08 Plan 02: WR-04 XzDecoder Multi-Stream Fix Summary

**Fixed silent data truncation in xz decompression by replacing XzDecoder::new() with new_multi_decoder(), enabling GNU-compatible concatenated .xz stream handling**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-28T23:49:36Z
- **Completed:** 2026-04-29T00:00:00Z
- **Tasks:** 1 (TDD: 2 commits — RED test + GREEN fix)
- **Files modified:** 2

## Accomplishments

- Replaced `XzDecoder::new(input)` with `XzDecoder::new_multi_decoder(input)` in `decompress_stream` — one-line fix that brings gow-xz into GNU compatibility for concatenated .xz files
- Added `concatenated_xz_streams_decompress_fully` integration test that builds two inline xz streams back-to-back and verifies both are decoded by `xz -d -c`
- Confirmed test fails before fix (only stream 1 decoded) and passes after fix (both streams decoded) — validates the fix is necessary and sufficient

## Task Commits

Each TDD phase was committed atomically:

1. **RED — failing test for WR-04** - `b81b76f` (test)
2. **GREEN — fix WR-04 implementation** - `5c87129` (feat)

## Files Created/Modified

- `crates/gow-xz/src/lib.rs` - Single-line change: `XzDecoder::new(input)` → `XzDecoder::new_multi_decoder(input)` in `decompress_stream` (line 86)
- `crates/gow-xz/tests/xz_tests.rs` - Appended `concatenated_xz_streams_decompress_fully` test using inline `XzEncoder` fixture construction (two concatenated streams)

## Decisions Made

- Single-line fix only — CONTEXT.md explicitly restricts gow-xz changes in phase 08 to WR-04 only; the existing `unwrap_or_else(|e| e.exit())` pattern on line 248 is not changed
- Inline fixture construction using `liblzma::write::XzEncoder` — consistent with D-02 (no binary fixture files committed to git)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. The RED/GREEN TDD cycle proceeded cleanly:
- RED: `concatenated_xz_streams_decompress_fully` failed with "stream 2 data missing from output" (confirming the bug)
- GREEN: All 8 gow-xz tests passed after the one-line fix

## Known Stubs

None. The fix is complete — no placeholder data, no TODOs in scope.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The change is limited to swapping one XzDecoder constructor for another; all I/O paths, file creation, and cleanup logic are unchanged (consistent with T-08-02-03 disposition: accept).

## Next Phase Readiness

- gow-xz fully fixed for WR-04; no follow-up needed
- Other phase 08 plans (08-01 gow-tar, 08-03 gow-gzip, 08-04 gow-curl) are independent and can proceed in parallel

---
*Phase: 08-code-review-fixes*
*Completed: 2026-04-29*
