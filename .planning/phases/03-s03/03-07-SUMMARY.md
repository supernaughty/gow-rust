---
phase: "03"
plan: "07"
---

# T07: Implement gow-cp with recursive copy, preservation, and symlink handling.

**Implement gow-cp with recursive copy, preservation, and symlink handling.**

## What Happened

Implemented `gow-cp` with full support for recursive copying, timestamp preservation, and symbolic link handling. The implementation uses `walkdir` for efficient filesystem traversal and `filetime` for accurate attribute preservation. Symlinks are handled according to GNU `cp` rules for `-P`, `-L`, and `-H` flags, with a fallback to Windows junctions when symlink creation privileges are missing (D-36). Verified the implementation with 16 integration tests covering happy paths, error cases, and Windows-specific behaviors.

## Verification

Ran 16 integration tests in `crates/gow-cp/tests/cp_tests.rs`. All tests passed, covering basic file copy, directory recursion, force overwrite, timestamp preservation, verbose output, and symlink dereference modes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-cp` | 0 | ✅ pass | 3500ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
