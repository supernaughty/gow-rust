---
phase: "04"
plan: "04"
---

# T04: Implement GNU-compatible sed with substitution and atomic in-place editing.

**Implement GNU-compatible sed with substitution and atomic in-place editing.**

## What Happened

I implemented `sed` in Rust, focusing on the substitution command (`s/find/replace/flags`) and reliable in-place editing (`-i`) for Windows.
Used `clap` for argument parsing, `regex` for pattern matching, and `gow_core::fs::atomic_rewrite` for safe in-place edits.
Implemented a BRE-to-ERE translation layer to support `\( \)` groups and literal characters as expected in standard `sed`.
Verified the implementation with 11 integration tests covering substitution, flags, capture groups, and in-place editing with backups.

## Verification

Built and ran integration tests covering substitution commands, flags, and in-place editing. All 11 tests passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-sed` | 0 | ✅ pass | 1440ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
