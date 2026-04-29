---
status: partial
phase: 11-new-utilities-wave2
source: [11-VERIFICATION.md]
started: 2026-04-30T00:00:00.000Z
updated: 2026-04-30T00:00:00.000Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. whoami runtime output
expected: Prints the current Windows username (single non-empty line), exits 0
result: [pending]

### 2. uname -r version correctness
expected: Output matches MAJOR.MINOR.BUILD with MAJOR >= 10 (e.g. "10.0.26200") — NOT "6.2" which would indicate GetVersionExW was used
result: [pending]

### 3. GNU compatibility spot-check — paste/join/split edge cases
expected: `paste - -` with stdin alternates lines correctly; `join` with mismatched field counts handles gracefully; `split -l 3` produces correct file chunks
result: [pending]

### 4. printf width/precision and expr exit codes
expected: `printf "%05.2f" 3.1` → "03.10"; `expr 3 - 3` exits 1 (not 0) and prints "0"
result: [pending]

### 5. [ bracket alias in installed PATH
expected: `[ -f somefile ]` works as an alias for test via extras/bin/[.bat → test.exe --_bracket_
result: [pending]

### 6. build.bat run — MSI staging regeneration
expected: Running `build.bat :run` regenerates CoreHarvest-x64.wxs including all 10 Phase 11 binaries (whoami.exe, uname.exe, paste.exe, join.exe, split.exe, printf.exe, expr.exe, test.exe, fmt.exe, unlink.exe)
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
