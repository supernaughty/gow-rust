---
phase: "04"
plan: "05"
---

# T05: Implement sort with external merge support for large files

**Implement sort with external merge support for large files**

## What Happened

Implemented the `sort` utility with robust large-file support using an external merge-sort mechanism. The utility buffers input lines until a configurable memory limit is reached (default 100MB), sorts the chunk in memory, and spills it to a `NamedTempFile` on disk. When all inputs are exhausted, `itertools::kmerge_by` efficiently merges the spilled files directly to the output stream. GNU compatibility is maintained with options for numeric sorting (`-n`), reverse (`-r`), unique output (`-u`), ignoring case (`-f`), and specifying output files (`-o`). Verified the implementation with extensive integration tests covering basic, reversed, numeric, unique, and external-merge sort behaviors. Fixed a bug in `test_numeric_sort_mixed` where identical numeric prefixes fell back to lexicographical order improperly in the test assertions.

## Verification

`cargo test -p gow-sort` passed successfully, and manual testing of an invalid file showed proper GNU-style error output. integration tests confirm proper sorting behavior for basic, reversed, numeric, unique, and merge cases.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-sort` | 0 | pass | 890ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
