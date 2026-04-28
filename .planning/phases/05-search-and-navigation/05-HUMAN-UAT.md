---
status: partial
phase: 05-search-and-navigation
source: [05-VERIFICATION.md]
started: 2026-04-28T04:00:00Z
updated: 2026-04-28T04:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Interactive scroll
expected: Viewport scrolls up and down; content updates on each keypress; no garbled output
result: [pending]

Run `cargo run -p gow-less -- <any large file>` in a real terminal. Press arrow keys, j/k, PgUp/PgDn, b/Space. Verify scrolling works correctly.

### 2. Search with n/N
expected: Viewport jumps to first match; n/N cycle through matches in forward and reverse order
result: [pending]

In the same session, type `/foo` (for some pattern that exists), press Enter. Then press n and N repeatedly.

### 3. g/G jump keys
expected: g sets viewport to line 0; G scans to EOF and positions at last line. Terminal remains responsive (no freeze on small files)
result: [pending]

Press g to jump to the top, then G to jump to the bottom of the file.

### 4. Clean exit / terminal restore
expected: Shell prompt returns normally with no broken cursor or raw-mode residue
result: [pending]

Press q. Verify the terminal is fully restored: prompt is on a clean line, echo is active, no stray artifacts.

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
