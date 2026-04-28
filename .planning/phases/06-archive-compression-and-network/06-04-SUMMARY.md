---
phase: 06-archive-compression-and-network
plan: "04"
subsystem: compression
tags: [liblzma, xz, unxz, lzma, compression, windows, msvc, static-linking]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    provides: "06-01 scaffold: gow-xz crate with liblzma dep declared and MSVC canary verified"

provides:
  - crates/gow-xz/src/lib.rs — full xz/unxz implementation using liblzma::write::XzEncoder and liblzma::read::XzDecoder
  - crates/gow-xz/tests/xz_tests.rs — 7 integration tests covering compress/decompress/roundtrip/binary/flags
  - R019 xz/unxz coverage complete

affects:
  - Phase 07+ (xz utility available for use in scripts and archives)

# Tech tracking
tech-stack:
  added:
    - liblzma::write::XzEncoder (compression at level 6, matches xz CLI default)
    - liblzma::read::XzDecoder (decompression)
  patterns:
    - argv[0] mode dispatch: invoked_as == "unxz" triggers decompress mode
    - compress_stream/decompress_stream as generic R: Read, W: Write helpers
    - -c/--stdout: pipe to stdout without removing original
    - -k/--keep: keep original after file-to-file operation
    - Per-file error handling: eprintln and continue (exit_code = 1) on error
    - Incomplete output file cleanup on error (fs::remove_file)

key-files:
  created:
    - crates/gow-xz/tests/xz_tests.rs
  modified:
    - crates/gow-xz/src/lib.rs

key-decisions:
  - "Use liblzma::write::XzEncoder (level 6) matching xz CLI default — not the read-side encoder variant"
  - "Decompress mode triggered by invoked_as == 'unxz' OR -d/--decompress flag"
  - "Remove incomplete output file on error to avoid leaving corrupt .xz files on disk"
  - "7 integration tests covering all specified behaviors plus edge cases"

patterns-established:
  - "xz/unxz mode: compress_stream wraps XzEncoder::new(output, 6) + encoder.finish(); decompress_stream wraps XzDecoder::new(input)"
  - "File naming: compress appends .xz; decompress strips .xz or errors (no silent ignore)"

requirements-completed:
  - R019

# Metrics
duration: 20min
completed: 2026-04-28
---

# Phase 06 Plan 04: xz/unxz Implementation Summary

**liblzma XzEncoder/XzDecoder wired into a full xz/unxz binary with argv[0] dispatch, file-to-file and stdout modes, -k keep flag, and 7 passing integration tests including binary round-trip.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-04-28T00:00:00Z
- **Completed:** 2026-04-28T00:20:00Z
- **Tasks:** 2
- **Files modified:** 2 (1 lib.rs rewrite, 1 tests replacement)

## Accomplishments

- Replaced gow-xz stub with 251-line full implementation using liblzma XzEncoder (level 6) and XzDecoder
- argv[0] mode dispatch: binary runs as "xz" (compress) or "unxz" (decompress) depending on invocation name
- -c/--stdout mode: write compressed/decompressed output to stdout, leave original intact
- -k/--keep mode: preserve original file after file-to-file operation
- Per-file error handling: print error to stderr and continue (GNU behavior)
- 7 integration tests covering all R019 behaviors plus binary content roundtrip

## Task Commits

1. **Task 1: Implement gow-xz lib.rs (xz / unxz)** - `792cd24` (feat)
2. **Task 2: Write integration tests for gow-xz** - `0b91aaa` (test)

## Files Created/Modified

- `crates/gow-xz/src/lib.rs` (251 lines) — Full xz/unxz implementation: Cli struct, Mode enum, compress_stream, decompress_stream, run(), uumain()
- `crates/gow-xz/tests/xz_tests.rs` (187 lines) — 7 integration tests: roundtrip, stdout mode, keep flag, decompress flag, invalid stream rejection, binary content roundtrip, missing extension error

## Decisions Made

- Used `liblzma::write::XzEncoder` (write-side) rather than `liblzma::read::XzEncoder` (read-side) — write-side matches the plan's specified API `XzEncoder::new(output, 6)` with `encoder.finish()`
- Compression level 6 matches the xz CLI default as specified
- Added incomplete output file cleanup on encode/decode error to prevent corrupt .xz files persisting on disk (Rule 2: missing critical behavior)
- 7 tests written (plan required minimum 5) — extra tests cover edge cases: missing .xz extension decompress and decompress-flag mode

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added incomplete output file cleanup on error**
- **Found during:** Task 1 (implementation)
- **Issue:** If encode or decode fails mid-stream, an incomplete output file would be left on disk. This is silent data corruption — a user would see a truncated/corrupt .xz file with no indication it's invalid.
- **Fix:** Added `fs::remove_file(&output_path)` in the error branch after a failed stream operation.
- **Files modified:** crates/gow-xz/src/lib.rs
- **Verification:** Behavior verified by xz_rejects_non_xz_file test (invalid input triggers error path)
- **Committed in:** 792cd24 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Essential for correctness. Prevents silent corrupt output files.

## Issues Encountered

None — liblzma MSVC canary from 06-01 confirmed the static feature compiles, so Task 1 proceeded without build failures.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- R019 xz/unxz coverage complete
- xz binary available for use in scripts
- cargo test -p gow-xz passes with 7 tests
- Wave 2 plan 06-04 complete; no blockers for remaining Wave 2 plans (06-02, 06-03, 06-05, 06-06)

## Self-Check

- crates/gow-xz/src/lib.rs: EXISTS (251 lines, min 100 required)
- crates/gow-xz/tests/xz_tests.rs: EXISTS (187 lines, min 70 required)
- grep "XzEncoder" crates/gow-xz/src/lib.rs: MATCH
- grep "XzDecoder" crates/gow-xz/src/lib.rs: MATCH
- grep "invoked_as.*unxz": MATCH (line 67)
- grep "try_convert_msys_path": MATCH (line 146)
- cargo test -p gow-xz: 7 passed, 0 failed
- Commit 792cd24: EXISTS
- Commit 0b91aaa: EXISTS

## Self-Check: PASSED

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*
