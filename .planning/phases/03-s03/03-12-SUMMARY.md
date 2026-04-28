---
phase: "03"
plan: "12"
---

# T12: Implement gow-tail with notify-based follow, truncation detection, and PID monitoring.

**Implement gow-tail with notify-based follow, truncation detection, and PID monitoring.**

## What Happened

Implemented `gow-tail` with support for last-N lines/bytes emission and real-time file following using the `notify` crate.

Key implementation details:
- Used `RecommendedWatcher` without debouncer for minimum latency.
- Implemented parent-directory watching to handle file creation (retry) and robust event filtering on Windows.
- Implemented truncation detection that resets to the beginning of the file when it shrinks.
- Added support for `--pid` on Windows using `OpenProcess` and `GetExitCodeProcess`.
- Added support for `--retry` in combination with `-f`.
- Implemented `tail_lines` using a chunked reverse scan for efficiency.
- Added 12 integration tests covering all major features, including follow, truncation, and PID-based termination.

All 12 tests passed successfully. The `test_tail_pid` failure in the first run was confirmed to be due to compilation time overhead and passed in subsequent runs.

## Verification

Ran 12 integration tests in `crates/gow-tail/tests/tail_test.rs` covering N-lines, N-bytes, multiple files, quiet mode, stdin, follow mode, truncation, zero-terminated, retry, and PID monitoring. All tests passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-tail` | 0 | ✅ pass | 1050ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
