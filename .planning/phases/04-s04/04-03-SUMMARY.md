---
phase: "04"
plan: "03"
---

# T03: Implement GNU-compatible grep with regex, recursion, and color support.

**Implement GNU-compatible grep with regex, recursion, and color support.**

## What Happened

Implemented `grep` utility for `gow-rust`. The implementation features:
- High-performance pattern matching using the `regex` crate (byte-searching).
- Recursive directory traversal using `walkdir`.
- Windows-friendly color output using `termcolor` and `gow-core` primitives.
- GNU-compatible argument parsing with `clap` and `gow_core::args::parse_gnu`, supporting flags: `-i` (ignore-case), `-v` (invert-match), `-r`/`-R` (recursive), `-n` (line-number), `-l` (files-with-matches), `-c` (count), `-h` (no-filename), `-H` (with-filename), `-E` (extended-regexp), `-F` (fixed-strings), and `--color`.
- Standard GNU exit codes: 0 for matches found, 1 for no matches, and 2 for errors.
- Comprehensive unit and integration tests covering various scenarios including stdin, multiple files, recursion, and error handling.
- UTF-8 console support via `gow_core::init()`.

Key implementation decisions:
- Used `regex::bytes::Regex` to handle arbitrary byte streams correctly, consistent with GNU grep.
- Used `bstr` for efficient line iteration without requiring valid UTF-8.
- Disabled `clap`'s default help/version flags to allow `-h` for `no-filename` and `-V` for potential future use (though currently using `--version`).
- Integrated with `gow_core` for consistent Windows-native behavior (VT mode, UTF-8).

## Verification

- cargo test -p gow-grep (Unit tests pass)
- cargo test -p gow-grep --test integration (12 integration tests pass)
- Manual verification of color output (via integration tests confirming ANSI codes are emitted when enabled).
- Verified exit codes match GNU convention.


## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-grep` | 0 | ✅ pass | 1500ms |
| 2 | `cargo test -p gow-grep --test integration` | 0 | ✅ pass | 1700ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
