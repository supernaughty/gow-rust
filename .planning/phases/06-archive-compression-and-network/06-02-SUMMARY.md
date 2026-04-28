---
phase: 06-archive-compression-and-network
plan: "02"
subsystem: compression
tags: [flate2, gzip, gunzip, zcat, GzEncoder, MultiGzDecoder, argv0-dispatch, assert_cmd]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    plan: "01"
    provides: crates/gow-gzip crate scaffold with flate2 dep declared, stub uumain

provides:
  - crates/gow-gzip/src/lib.rs — full gzip/gunzip/zcat implementation with GzEncoder/MultiGzDecoder
  - crates/gow-gzip/src/gunzip.rs — gunzip binary entry point (argv[0] dispatches to lib.rs)
  - crates/gow-gzip/src/zcat.rs — zcat binary entry point (argv[0] dispatches to lib.rs)
  - crates/gow-gzip/tests/gzip_tests.rs — 8 integration tests covering R019 gzip coverage

affects:
  - 06-05-PLAN (gow-tar needs gzip support wired via gow-gzip for -z flag understanding)

# Tech tracking
tech-stack:
  added: []  # flate2 was already declared in Wave 1; no new deps
  patterns:
    - argv[0] mode dispatch: detect_mode(invoked_as, decompress_flag) via Path::file_stem().to_lowercase()
    - GzEncoder wraps Write: GzEncoder::new(output, Compression::default()), io::copy, encoder.finish()
    - MultiGzDecoder wraps Read: MultiGzDecoder::new(input), io::copy to output (handles concatenated streams)
    - Per-file error handling: eprintln! and accumulate exit_code=1, never early-return from file loop
    - Pitfall 4 guard: stdin decompress error emits "not in gzip format" and returns 1 instead of passing raw bytes
    - Three binaries from one crate: gzip (main.rs), gunzip (gunzip.rs), zcat (zcat.rs) — all call uu_gzip::uumain

key-files:
  created:
    - crates/gow-gzip/src/gunzip.rs
    - crates/gow-gzip/src/zcat.rs
  modified:
    - crates/gow-gzip/src/lib.rs (stub replaced with full implementation)
    - crates/gow-gzip/Cargo.toml (added [[bin]] entries for gunzip and zcat)
    - crates/gow-gzip/tests/gzip_tests.rs (Wave 0 placeholder replaced with 8 integration tests)

key-decisions:
  - "Three [[bin]] entries in one Cargo.toml (gzip, gunzip, zcat) rather than three separate crates — all share lib.rs via argv[0] dispatch; matches GNU gzip architecture where gunzip is a symlink"
  - "MultiGzDecoder for all decompress paths (not GzDecoder) — handles real-world concatenated gzip streams; zcat in particular needs this"
  - "Pitfall 4 guard implemented: stdin decompress failure prints 'not in gzip format' and exits 1 — stdin passthrough of raw bytes is a security concern"
  - "stdout mode (-c/zcat) never removes input file even without -k flag"

patterns-established:
  - "Three-binary-one-crate pattern: add extra [[bin]] sections with separate src/name.rs entry points; all delegate to lib uumain"
  - "Gzip file naming: compress appends .gz, decompress strips .gz; fallback to .out suffix for non-.gz decompress input"
  - "Partial output cleanup: on error during file-to-file compress/decompress, remove the incomplete output file before returning"

requirements-completed:
  - R019

# Metrics
duration: 20min
completed: 2026-04-28
---

# Phase 06 Plan 02: gzip/gunzip/zcat Implementation Summary

**gzip/gunzip/zcat implemented with flate2 GzEncoder/MultiGzDecoder, argv[0] dispatch, and 8 integration tests covering compress/decompress round-trip, stdout mode, keep flag, and error handling.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-04-28T00:00:00Z
- **Completed:** 2026-04-28T00:20:00Z
- **Tasks:** 2
- **Files modified:** 5 (3 modified + 2 created)

## Accomplishments

- Replaced Wave 1 stub in crates/gow-gzip/src/lib.rs with full gzip/gunzip/zcat implementation using flate2 1.1
- Added gunzip and zcat as separate `[[bin]]` entries in Cargo.toml with dedicated entry point files (gunzip.rs, zcat.rs) — all three binaries share the same lib.rs via argv[0] mode detection
- 8 integration tests cover: round-trip compression/decompression, stdout mode (-c), keep flag (-k), zcat stdout output, gzip -d flag, invalid input rejection, stdin compression, and multi-file keep flag
- All tests pass: `cargo test -p gow-gzip` exits 0 with 8 tests

## Task Commits

1. **Task 1: Implement gow-gzip lib.rs (gzip / gunzip / zcat)** - `5948be4` (feat)
2. **Task 2: Write integration tests for gow-gzip** - `d0f29b6` (test)

## Files Created/Modified

- `crates/gow-gzip/src/lib.rs` — Full implementation: Mode enum, detect_mode/is_stdout_mode, compress_stream (GzEncoder), decompress_stream (MultiGzDecoder), run() file loop with per-file error handling, uumain with argv[0] dispatch
- `crates/gow-gzip/src/gunzip.rs` — 3-line binary entry point for gunzip binary
- `crates/gow-gzip/src/zcat.rs` — 3-line binary entry point for zcat binary
- `crates/gow-gzip/Cargo.toml` — Added `[[bin]]` sections for gunzip and zcat
- `crates/gow-gzip/tests/gzip_tests.rs` — 8 integration tests replacing Wave 0 placeholder

## Decisions Made

- **Three [[bin]] in one crate:** Added gunzip.rs and zcat.rs as binary entry points in the same crate rather than creating separate crates. This matches the GNU gzip architecture (gunzip is historically a symlink/hardlink to gzip). All three binaries call the same `uu_gzip::uumain` and mode dispatch happens via argv[0] file stem.
- **MultiGzDecoder not GzDecoder:** Used `MultiGzDecoder` for all decompress paths to handle concatenated gzip streams (common in real-world data pipelines). This is especially important for `zcat` which is often used to concatenate compressed streams.
- **Pitfall 4 guard:** The RESEARCH.md pitfall for stdin decompression is implemented: when stdin decompress fails, emit "gzip: stdin: not in gzip format" and exit 1. Raw bytes are never passed through.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added gunzip and zcat [[bin]] entries to Cargo.toml**
- **Found during:** Task 1 (implementation) / Task 2 (test authoring)
- **Issue:** The plan specifies tests using `Command::cargo_bin("gunzip")` and `Command::cargo_bin("zcat")`, but the Wave 1 Cargo.toml only had a single `[[bin]] name = "gzip"` entry. Without gunzip and zcat binaries, the integration tests would fail with "binary not found".
- **Fix:** Created `src/gunzip.rs` and `src/zcat.rs` as 3-line binary entry points (each calls `uu_gzip::uumain`), and added `[[bin]]` sections for both in Cargo.toml. The plan's note at the bottom of Task 2 said "if cargo_bin('gunzip') fails because only 'gzip' binary is built, use `gzip -d` as the decompress command" — but adding the binaries is the better approach and matches GNU gzip behavior.
- **Files modified:** `crates/gow-gzip/Cargo.toml`, `crates/gow-gzip/src/gunzip.rs`, `crates/gow-gzip/src/zcat.rs`
- **Verification:** `cargo test -p gow-gzip` passes all 8 tests including gunzip and zcat binary tests
- **Committed in:** `5948be4` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - missing critical binary targets)
**Impact on plan:** The fix is required for the tests to work as specified. No scope creep.

## Known Stubs

None — all stub behavior from Wave 1 has been replaced with full implementation.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes beyond what the plan's threat model covers.

| Flag | File | Description |
|------|------|-------------|
| T-06-02-01 (mitigated) | crates/gow-gzip/src/lib.rs | Output path derived from input path with suffix add/strip — no path traversal; no output path from archive metadata |
| T-06-02-04 (mitigated) | crates/gow-gzip/src/lib.rs | Stdin Pitfall 4 guard implemented: decompress error → "not in gzip format" + exit 1 |

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- gow-gzip crate is fully functional: gzip, gunzip, zcat binaries all built and tested
- R019 gzip/gunzip/zcat coverage satisfied
- Wave 2 parallel plans (06-03 bzip2, 06-04 xz, 06-05 tar, 06-06 curl) can proceed independently
- No blockers

## Self-Check: PASSED

- `crates/gow-gzip/src/lib.rs` — 220 lines (min 120 required) — FOUND
- `crates/gow-gzip/tests/gzip_tests.rs` — 256 lines (min 80 required) — FOUND
- `crates/gow-gzip/src/gunzip.rs` — FOUND
- `crates/gow-gzip/src/zcat.rs` — FOUND
- Commit `5948be4` (Task 1 feat) — FOUND in git log
- Commit `d0f29b6` (Task 2 test) — FOUND in git log
- `cargo test -p gow-gzip` — 8 tests passed, 0 failed
- `grep "GzEncoder" crates/gow-gzip/src/lib.rs` — MATCH
- `grep "MultiGzDecoder" crates/gow-gzip/src/lib.rs` — MATCH
- `grep "invoked_as" crates/gow-gzip/src/lib.rs` — MATCH
- `grep "try_convert_msys_path" crates/gow-gzip/src/lib.rs` — MATCH
- `grep "follow_symlinks\|tokio\|async" crates/gow-gzip/src/lib.rs` — NO MATCH (correct)

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*
