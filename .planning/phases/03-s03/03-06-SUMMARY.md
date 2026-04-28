---
phase: "03"
plan: "06"
---

# T06: Implement gow-unix2dos with shared transform logic and 12 integration tests.

**Implement gow-unix2dos with shared transform logic and 12 integration tests.**

## What Happened

Implemented `gow-unix2dos` by reusing the `transform` module from `gow-dos2unix`. The implementation supports in-place conversion using `gow_core::fs::atomic_rewrite`, `-n` for new-file output, and `-k` for timestamp preservation. 12 integration tests were added, including a round-trip test that verifies `unix2dos` and `dos2unix` work together correctly. Clippy is clean and all tests pass.

## Verification

Ran `cargo test -p gow-unix2dos` (12 tests passed) and `cargo clippy`. Verified round-trip conversion with `dos2unix`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-unix2dos` | 0 | ✅ pass | 1900ms |
| 2 | `cargo clippy -p gow-unix2dos --all-targets -- -D warnings` | 0 | ✅ pass | 6080ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
