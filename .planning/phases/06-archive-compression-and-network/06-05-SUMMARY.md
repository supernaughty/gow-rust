---
phase: 06-archive-compression-and-network
plan: "05"
subsystem: archive, compression
tags: [tar, flate2, bzip2, gzip, archive, windows]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    provides: scaffold crates with deps declared (tar, flate2, bzip2, walkdir in gow-tar/Cargo.toml)

provides:
  - crates/gow-tar/src/lib.rs: full tar implementation with -c/-x/-t modes and -z/-j codec flags
  - crates/gow-tar/tests/tar_tests.rs: 8 integration tests covering R018

affects:
  - 06-06-PLAN (gow-curl — no overlap, Wave 2 parallel)
  - Phase 07+ (any plan that creates/extracts tar archives)

# Tech tracking
tech-stack:
  added:
    - tar = "0.4" (Builder + Archive API, append_dir_all, follow_symlinks, Header::new_gnu)
    - flate2 = "1.1" (GzEncoder + GzDecoder streaming; already declared in workspace)
    - bzip2 = "0.6" (BzEncoder + BzDecoder streaming; already declared in workspace)
  patterns:
    - Closure-based finisher pattern: append_paths<W,F>(builder, cli, finish) avoids Finishable trait issues
    - -C directory resolution: resolve path args relative to -C base in create mode
    - Per-entry unpack_in() with graceful symlink error handling (T-06-05-02 mitigation)
    - follow_symlinks(false) mandatory call pattern (GNU tar behavior, NOT tar crate default)
    - Header::new_gnu() for all headers (GNU @LongLink for paths > 100 chars)

key-files:
  created:
    - crates/gow-tar/tests/tar_tests.rs (8 integration tests, 348 lines)
  modified:
    - crates/gow-tar/src/lib.rs (full implementation replacing stub, 394 lines)

key-decisions:
  - "Use closure-based finisher in append_paths helper instead of Finishable trait to avoid generic bound conflicts"
  - "Implement -C directory resolution: path args are joined with base_dir when -C is given (GNU tar behavior)"
  - "Use per-entry unpack_in() loop instead of archive.unpack() to handle symlink errors gracefully"
  - "follow_symlinks(false) is called BEFORE the path loop — must not be skipped even on empty paths list"
  - "Header::new_gnu() selected over Header::new_ustar() to support paths > 100 bytes (T-06-05-05)"

patterns-established:
  - "Codec dispatch pattern: Codec enum (Plain/Gzip/Bzip2) matched in create/extract/list functions"
  - "Mode detection: detect_mode() counts how many of -c/-x/-t are set; errors on 0 or >1"
  - "GNU tar -C flag: create mode resolves positional path args relative to -C directory"

requirements-completed:
  - R018

# Metrics
duration: 30min
completed: 2026-04-28
---

# Phase 06 Plan 05: GNU tar — Create/Extract/List with gzip and bzip2 Codec Summary

**tar -c/-x/-t with -z/-j codec flags implemented via tar 0.4 + flate2 + bzip2 crates; 8 integration tests cover all R018 round-trips; follow_symlinks(false) explicitly enforced.**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-04-28T14:00:00Z
- **Completed:** 2026-04-28T14:24:35Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Replaced stub `uumain` in `crates/gow-tar/src/lib.rs` with full 394-line GNU tar implementation
- Implemented all three modes: `-c` (create), `-x` (extract), `-t` (list)
- Implemented all three codecs: plain tar, gzip (`-z` via flate2), bzip2 (`-j` via bzip2)
- CRITICAL: `builder.follow_symlinks(false)` called explicitly — tar crate default is `true` (dereference), GNU tar default is `false` (store symlink)
- `Header::new_gnu()` used for all entries (GNU @LongLink extension, no 100-byte truncation)
- `-C <dir>` flag correctly resolves path args relative to the specified directory in create mode
- Graceful symlink extraction error handling on Windows (T-06-05-02: log warning, continue)
- `unpack_in()` used per-entry (path traversal guard built-in — T-06-05-01 mitigation)
- 8 integration tests added; all pass via `cargo test -p gow-tar`

## Task Commits

1. **Task 1: Implement gow-tar lib.rs (create/extract/list with -z/-j codec)** - `d0ec2ce` (feat)
2. **Task 2: Write integration tests + fix -C directory handling** - `783fe3b` (test)

## Files Created/Modified

- `crates/gow-tar/src/lib.rs` — Full tar implementation (394 lines): Cli struct, Mode/Codec enums, detect_mode/detect_codec, run_create/run_extract/run_list, append_paths helper with closure finisher, unpack_archive with per-entry symlink handling
- `crates/gow-tar/tests/tar_tests.rs` — 8 integration tests (348 lines): create_and_extract_tar_gz, create_and_extract_tar_bz2, create_and_extract_plain_tar, list_tar_gz, list_plain_tar, roundtrip_preserves_content, create_directory_archive, error_when_no_mode_specified

## Decisions Made

- **Closure-based finisher:** The initial implementation used a `Finishable` trait on the generic `W: Write` type parameter of `append_paths`. This failed to compile because `Builder::into_inner()` returns `W` without the `Finishable` bound. Switched to a closure `F: FnOnce(W) -> Result<()>` that each codec arm provides inline — cleaner and no extra trait required.
- **Per-entry unpack_in() loop for extract:** Rather than `archive.unpack(dest)`, using a per-entry loop with `entry.unpack_in(dest)` allows per-entry error handling. The `unpack_in()` method retains the same path traversal guard as `unpack()`.
- **-C directory resolution in create mode:** GNU tar resolves positional path arguments relative to `-C <dir>`. The implementation joins `base_dir + path_str` when `-C` is given. The archive entry name is still the bare `path_str` (e.g. `"src"`), not the full resolved path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed -C directory flag not resolving path args in create mode**

- **Found during:** Task 2 (integration tests — create_and_extract_tar_gz failing)
- **Issue:** When `tar -c -C /some/dir src` was run, the implementation tried to archive `Path::new("src")` directly (which doesn't exist in the cwd), instead of resolving it as `/some/dir/src`. GNU tar changes the working directory for path resolution when `-C` is given.
- **Fix:** In `append_paths`, compute `full_path = base_dir.join(candidate)` when `-C` is specified. The archive entry name remains the original `path_str` (bare "src"), matching GNU tar output format.
- **Files modified:** `crates/gow-tar/src/lib.rs`
- **Verification:** `cargo test -p gow-tar` — all 8 tests pass including all round-trip tests
- **Committed in:** `783fe3b` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** The -C fix is necessary for correct operation. All round-trip tests were blocked until this was fixed. No scope creep.

## Threat Model Verification

| Threat ID | Status | Evidence |
|-----------|--------|---------|
| T-06-05-01 (tar slip) | Mitigated | `entry.unpack_in(dest)` — path traversal guard built-in |
| T-06-05-02 (symlink privilege) | Accepted | Warning logged, extraction continues |
| T-06-05-03 (DoS via entries) | Accepted | entries() iterator — no Vec buffering |
| T-06-05-04 (overwrite on extract) | Accepted | unpack_in() overwrites by default (GNU tar default) |
| T-06-05-05 (long paths truncation) | Mitigated | Header::new_gnu() used — @LongLink extension |

## Issues Encountered

- Finishable trait approach for the generic write helper failed to compile (see Decisions Made). Resolved by switching to a closure-based finisher.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None. The stub from Wave 0 (`tar: not implemented`) has been fully replaced with a working implementation. All 8 integration tests pass and exercise all modes and codecs.

## Next Phase Readiness

- `crates/gow-tar` is fully implemented with all R018 requirements satisfied
- Wave 2 parallel plans (06-02 gzip, 06-03 bzip2, 06-04 xz, 06-06 curl) run in parallel — no overlap
- R018 complete: `tar -czf`, `tar -xzf`, `tar -tzf`, `tar -cjf`, `tar -xjf`, `tar -cf`, `tar -xf` all tested and passing

## Self-Check

- [x] `crates/gow-tar/src/lib.rs` exists and is 394 lines (min 180 required)
- [x] `crates/gow-tar/tests/tar_tests.rs` exists and is 348 lines (min 100 required)
- [x] Commits `d0ec2ce` and `783fe3b` present in git log
- [x] `cargo test -p gow-tar` exits 0 with 8 tests passed
- [x] `grep "follow_symlinks(false)" crates/gow-tar/src/lib.rs` returns match
- [x] `grep "follow_symlinks(true)" crates/gow-tar/src/lib.rs` returns no match
- [x] `grep "Header::new_gnu" crates/gow-tar/src/lib.rs` returns match

## Self-Check: PASSED

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*
