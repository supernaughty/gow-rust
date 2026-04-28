---
phase: 06-archive-compression-and-network
plan: "03"
subsystem: compression
tags: [bzip2, bunzip2, MultiBzDecoder, BzEncoder, compression, archive, windows, rust]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    provides: gow-bzip2 crate scaffold (Wave 1) with bzip2 dep declared and stub uumain

provides:
  - crates/gow-bzip2/src/lib.rs — full bzip2/bunzip2 implementation with argv[0] dispatch
  - crates/gow-bzip2/tests/bzip2_tests.rs — 8 integration tests covering R019

affects:
  - 06-05-PLAN (gow-tar needs bzip2 codec for -j flag; can import uu_bzip2 or use bzip2 crate directly)

# Tech tracking
tech-stack:
  added: []  # bzip2 0.6 already declared in workspace by Wave 1 scaffold
  patterns:
    - "argv[0] mode dispatch: detect invoked_as == 'bunzip2' via Path::file_stem().to_lowercase()"
    - "MultiBzDecoder for decompress: handles multi-stream .bz2 files (pbzip2, Wikipedia dumps)"
    - "BzEncoder::new(output, Compression::default()) + encoder.finish() for compress"
    - "stdout_mode implies keep=true: -c preserves original, routes output to io::stdout()"
    - "Per-file error handling: eprintln! + exit_code=1 + continue (never early-return from loop)"
    - "Output path derivation from input path: compress adds .bz2, decompress strips .bz2"

key-files:
  created:
    - crates/gow-bzip2/tests/bzip2_tests.rs
  modified:
    - crates/gow-bzip2/src/lib.rs

key-decisions:
  - "Use MultiBzDecoder (not BzDecoder) for decompress path — handles real-world multi-stream .bz2 files from pbzip2 and Wikipedia dumps"
  - "No bunzip2 separate binary entry in Cargo.toml — mode dispatches via argv[0] check in uumain; tests use bzip2 -d for decompress"
  - "stdout_mode (-c) implies keep=true implicitly — avoid deleting original when output goes to stdout"
  - "Partial output cleanup on error: remove .bz2/.txt partial file if compress/decompress fails mid-stream"

patterns-established:
  - "bzip2/bunzip2 argv[0] dispatch: identical shape to gow-gzip gunzip dispatch, swap codec types"
  - "Integration test pattern: write_fixture() helper + tempdir + assert_cmd::Command::cargo_bin"

requirements-completed:
  - R019

# Metrics
duration: 15min
completed: 2026-04-28
---

# Phase 06 Plan 03: bzip2/bunzip2 Implementation Summary

**Full bzip2/bunzip2 using MultiBzDecoder + BzEncoder with argv[0] dispatch, -d/-c/-k flags, and 8 round-trip integration tests — R019 bzip2 coverage complete.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-28T14:06:55Z
- **Completed:** 2026-04-28T14:21:55Z
- **Tasks:** 2
- **Files modified:** 2 (1 created + 1 replaced)

## Accomplishments

- Replaced Wave 1 stub with full bzip2/bunzip2 implementation using `MultiBzDecoder` and `BzEncoder`
- Mode dispatch: `bunzip2` via argv[0] OR `-d`/`--decompress` flag, using `Path::file_stem().to_lowercase()`
- Flags implemented: `-c`/`--stdout` (stdout mode), `-k`/`--keep` (preserve original), `-d`/`--decompress`
- 8 integration tests covering round-trip, stdout mode, keep flag, -d flag, error cases, and multiple files
- All tests pass: `cargo test -p gow-bzip2` exits 0 with 8 tests passing

## Task Commits

1. **Task 1: Implement gow-bzip2 lib.rs (bzip2/bunzip2)** - `d04cca3` (feat)
2. **Task 2: Write integration tests for gow-bzip2** - `2388320` (test)

**Plan metadata:** (docs commit below)

## Files Created/Modified

- `crates/gow-bzip2/src/lib.rs` — Full implementation: Cli struct, Mode enum, compress_stream(), decompress_stream(), run(), uumain()
- `crates/gow-bzip2/tests/bzip2_tests.rs` — 8 integration tests: compress_decompress_roundtrip, compress_to_stdout, keep_flag_preserves_original, decompress_flag, decompress_to_stdout, decompress_file_without_bz2_suffix_fails, decompress_non_bzip2_data_fails, compress_multiple_files

## Decisions Made

- **MultiBzDecoder for all decompress paths:** The plan explicitly requires this for real-world multi-stream .bz2 compatibility (pbzip2, Wikipedia dumps). `BzDecoder` (single-stream) is not used anywhere in the implementation.
- **No separate bunzip2 binary:** Cargo.toml from Wave 1 only declares one `[[bin]]` named `bzip2`. Mode is dispatched via argv[0] check. Integration tests use `bzip2 -d` for decompress operations instead of `Command::cargo_bin("bunzip2")`.
- **stdout_mode implies keep:** When `-c` is passed, the original file is always preserved regardless of `-k` flag state. This matches GNU bzip2 behavior where `-c` routes output to stdout without touching the input file.
- **Partial output cleanup:** If compression or decompression fails mid-stream, the partially written output file is removed. This prevents corrupt output files being left on disk.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `crates/gow-bzip2` is fully implemented and all tests pass
- gow-tar (06-05) can use `bzip2` workspace crate directly with `BzEncoder`/`BzDecoder` for `-j` flag — the gow-bzip2 lib.rs implementation is also a reference pattern for gow-tar's bzip2 codec path
- R019 bzip2 coverage complete; only gow-gzip (R019 gzip) and gow-xz (R019 xz) remain for full R019 satisfaction

## Self-Check: PASSED

- `crates/gow-bzip2/src/lib.rs` — verified present, 284 lines, exports `uumain`
- `crates/gow-bzip2/tests/bzip2_tests.rs` — verified present, 199 lines, contains `roundtrip`
- Commit `d04cca3` — verified in git log (feat: bzip2/bunzip2 implementation)
- Commit `2388320` — verified in git log (test: 8 integration tests)
- `cargo test -p gow-bzip2` — 8 tests pass, 0 failed
- `grep "MultiBzDecoder" crates/gow-bzip2/src/lib.rs` — returns matches (line 12, 75, 77)
- `grep "BzDecoder[^)]" crates/gow-bzip2/src/lib.rs` — no single-stream BzDecoder usage

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*
