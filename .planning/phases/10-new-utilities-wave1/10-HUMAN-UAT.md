---
status: partial
phase: 10-new-utilities-wave1
source: [10-VERIFICATION.md]
started: 2026-04-29T00:00:00Z
updated: 2026-04-29T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. GNU reference comparison
expected: Output of each utility matches actual GNU coreutils (Linux/WSL) on the same input — tests are not merely self-consistent, expected values are verified against reference implementation
result: [pending]

### 2. du/df on hardware edge cases
expected: df silently skips optical drives with no media inserted; du handles deep symlink trees without following links
result: [pending]

### 3. md5sum/sha1sum/sha256sum -c round-trip
expected: Generate a checkfile on Windows, verify it with -c, then confirm -c exits non-zero when a file is modified
result: [pending]

### 4. MSI inclusion — build.bat :run staging
expected: Running `build.bat :run` copies all 13 new binaries (seq, sleep, tac, nl, od, fold, expand, unexpand, du, df, md5sum, sha1sum, sha256sum) into the staging output alongside existing utilities
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
