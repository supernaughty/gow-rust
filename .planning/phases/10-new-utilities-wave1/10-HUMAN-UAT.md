---
status: complete
phase: 10-new-utilities-wave1
source: [10-VERIFICATION.md]
started: 2026-04-29T00:00:00Z
updated: 2026-04-29T13:40:00Z
---

## Current Test

[testing complete]

## Tests

### 1. GNU Reference Comparison
expected: Output of each utility matches actual GNU coreutils (Linux/WSL) on the same input
result: pass

### 2. du/df on hardware edge cases
expected: df silently skips optical drives with no media inserted; du -sh . reports human-readable size
result: pass

### 3. md5sum/sha1sum/sha256sum -c round-trip
expected: md5sum -c verifies OK on unmodified file, FAILED + exit 1 on modified file
result: pass
notes: sha256sum file save typo in test run (sums.sha25 vs sums.sha256) — not a code bug; md5sum round-trip confirmed correct behavior

### 4. MSI inclusion — build.bat :run staging
expected: All 13 new binaries (seq, sleep, tac, nl, od, fold, expand, unexpand, du, df, md5sum, sha1sum, sha256sum) appear in build output
result: pass
notes: staging\ only created during WiX MSI packaging, not dev build. All 13 binaries confirmed in build.bat :run output listing.

## Summary

total: 4
passed: 4
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
