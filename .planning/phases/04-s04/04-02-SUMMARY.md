---
phase: "04"
plan: "02"
---

# T02: Implement GNU-compatible tr, cut, and uniq stream filters with Windows UTF-8 support.

**Implement GNU-compatible tr, cut, and uniq stream filters with Windows UTF-8 support.**

## What Happened

I have implemented the three stream filter utilities: `tr`, `cut`, and `uniq`. 

1. **`tr` (translate)**:
   - Supports character translation, deletion (`-d`), squeezing (`-s`), and complement (`-c`).
   - Implemented basic set expansion for character ranges (`a-z`) and escapes (octal `\012`, etc.).
   - Verified with unit tests for set expansion and integration tests for all major modes.

2. **`cut` (remove sections)**:
   - Supports byte-based (`-b`), character-based (`-c`), and field-based (`-f`) selection.
   - Character mode is Unicode-aware using `bstr`, matching the project's UTF-8 console policy.
   - Supports custom delimiters (`-d`), output delimiters (`--output-delimiter`), and complement selection (`--complement`).
   - Verified with unit tests for range parsing and integration tests for bytes, fields, complement, and Unicode characters.

3. **`uniq` (report/omit repeated lines)**:
   - Supports basic duplicate removal, count prefixing (`-c`), only duplicates (`-d`), and only unique lines (`-u`).
   - Implemented field skipping (`-f`), character skipping (`-s`), and character checking limit (`-w`).
   - Supports case-insensitive comparison (`-i`) and zero-terminated lines (`-z`).
   - Verified with unit tests for line comparison logic and integration tests for basic usage, counting, and repeated lines.

All utilities are integrated into the `gow-rust` workspace and call `gow_core::init()` to ensure Windows-native UTF-8 console support. Tests pass for all three crates.

## Verification

Ran `cargo test -p gow-tr -p gow-cut -p gow-uniq` which includes unit tests and integration tests for all implemented features.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-tr -p gow-cut -p gow-uniq` | 0 | ✅ pass | 3500ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
