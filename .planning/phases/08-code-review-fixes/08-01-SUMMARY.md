---
phase: 08-code-review-fixes
plan: "01"
subsystem: gow-tar
tags: [bug-fix, bzip2, multi-stream, cli-error-handling, exit-codes, tar]
dependency_graph:
  requires: []
  provides: [FIX-01, FIX-02, FIX-03]
  affects: [crates/gow-tar/src/lib.rs, crates/gow-tar/tests/tar_tests.rs]
tech_stack:
  added: []
  patterns: [MultiBzDecoder, had_error propagation, graceful CLI error match]
key_files:
  created: []
  modified:
    - crates/gow-tar/src/lib.rs
    - crates/gow-tar/tests/tar_tests.rs
decisions:
  - "Added set_ignore_zeros(true) to Archive in unpack_archive: required for tar crate to read past end-of-archive zero blocks when MultiBzDecoder provides a concatenated byte stream from multiple bzip2 streams"
  - "Test arg format: changed 'xjf'/'xzf' combined flag strings to separate '-x -j -f'/'-x -z -f' args because gow-tar uses clap which requires separate flags (not GNU-style combined short flags)"
metrics:
  duration: "4 minutes"
  completed: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 08 Plan 01: Fix WR-01, WR-02, WR-03 in gow-tar Summary

**One-liner:** Fixed multi-stream bzip2 extraction (MultiBzDecoder + set_ignore_zeros), graceful CLI error (exit 2), and had_error propagation (exit 1 on partial failure) in gow-tar.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix WR-01 (MultiBzDecoder) and WR-02 (graceful CLI error) | 88c7f46 | crates/gow-tar/src/lib.rs |
| 2 | Fix WR-03 (had_error propagation) and add tests for WR-01/02/03 | f0071b8 | crates/gow-tar/src/lib.rs, crates/gow-tar/tests/tar_tests.rs |

## What Was Built

### WR-01: MultiBzDecoder (multi-stream bzip2 extraction)

Replaced `bzip2::read::BzDecoder` with `bzip2::read::MultiBzDecoder` at all 4 call sites in `run_extract` (2 sites) and `run_list` (2 sites). `MultiBzDecoder` transparently reads multiple concatenated bzip2 streams, where `BzDecoder` would stop after the first stream — silently dropping archive entries.

Also added `archive.set_ignore_zeros(true)` to `unpack_archive` so the tar crate continues reading past end-of-archive zero blocks when extracting from concatenated archives. Without this, even with `MultiBzDecoder`, the tar reader would stop at the first archive's EOF marker.

### WR-02: Graceful CLI Error (exit 2)

Replaced `Cli::from_arg_matches(&matches).unwrap()` in `uumain` with a match expression that:
- Returns `Ok(cli)` on success
- Prints `tar: {e}` to stderr and returns exit code 2 on failure

This eliminates the panic behavior when clap encounters argument parse errors.

### WR-03: had_error Propagation (exit 1 on partial extraction)

Replaced the `unpack_archive` function body with `had_error` tracking:
- `let mut had_error = false;` initialized before the entry loop
- Symlink/privilege/access-denied errors: emit warning to stderr, keep `had_error = false` (per D-06 caveat)
- All other errors: emit error to stderr, set `had_error = true`
- After the loop: `if had_error { anyhow::bail!("one or more files could not be extracted"); }`
- The existing `uumain` match maps `Err(_)` → exit code 1, so had_error → bail! → exit 1 is automatic

### Tests Added (tar_tests.rs)

Three new integration tests appended (existing 8 tests preserved):

- `multi_stream_bzip2_extracts_both_entries` (WR-01): Builds two separate bzip2-compressed tar archives inline, concatenates them, verifies `tar -x -j -f` extracts both `file_a.txt` and `file_b.txt`
- `missing_mode_flag_exits_with_error` (WR-02): Verifies `tar somefile.tar` (no -c/-x/-t) exits non-zero with "option" or "cxt" in stderr
- `extraction_failure_exits_nonzero` (WR-03): Creates a valid .tar.gz archive, attempts extraction into a nonexistent directory, verifies exit code is non-zero

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test argument format (combined flag strings)**
- **Found during:** Task 2 — first test run
- **Issue:** Plan spec used `"xjf"` and `"xzf"` as single positional args. The gow-tar binary uses clap which parses these as positional path arguments, not combined short flags. The command exited with code 2 ("You must specify one of the '-cxt' options").
- **Fix:** Changed to separate flags: `"-x", "-j", "-f"` and `"-x", "-z", "-f"` in both affected tests.
- **Files modified:** crates/gow-tar/tests/tar_tests.rs
- **Commit:** f0071b8

**2. [Rule 2 - Missing Critical Functionality] Added set_ignore_zeros(true) to Archive**
- **Found during:** Task 2 — multi_stream_bzip2 test failing (file_b.txt missing)
- **Issue:** Even with `MultiBzDecoder` decoding both bzip2 streams into a single byte stream, the tar crate's `Archive::entries()` stops at the end-of-archive zero blocks from the first tar archive. Both fixes are required together for multi-stream extraction to work.
- **Fix:** Added `archive.set_ignore_zeros(true)` at the start of `unpack_archive`. This is documented in the tar crate specifically for concatenated-archive use cases.
- **Files modified:** crates/gow-tar/src/lib.rs
- **Commit:** f0071b8

**3. [Rule 1 - Bug] Removed unused `std::io::Write` import from test file**
- **Found during:** Task 2 — compiler warning
- **Issue:** Plan spec included `use std::io::Write;` in the test imports but the test functions don't use `Write` directly (the BzEncoder uses it internally).
- **Fix:** Removed unused import to eliminate compiler warning.
- **Files modified:** crates/gow-tar/tests/tar_tests.rs
- **Commit:** f0071b8

## Known Stubs

None. All fixes are complete implementations with passing tests.

## Verification Results

```
cargo build -p gow-tar  → Finished (no errors)
cargo test -p gow-tar   → 11 passed; 0 failed; 0 ignored
```

Specific checks:
- `grep -n "BzDecoder::new"` → all 4 occurrences are `MultiBzDecoder::new`
- `grep -n "MultiBzDecoder"` → 5 lines (1 import + 4 call sites)
- `grep -n "had_error"` → 4 lines (declaration, comment, set=true, if-check)
- `grep -n "anyhow::bail!"` → 1 line containing "could not be extracted"
- `grep -n "return 2"` → 2 lines in the Cli error and detect_mode Err arms

## Self-Check: PASSED

- crates/gow-tar/src/lib.rs: FOUND
- crates/gow-tar/tests/tar_tests.rs: FOUND
- Commit 88c7f46: FOUND
- Commit f0071b8: FOUND
