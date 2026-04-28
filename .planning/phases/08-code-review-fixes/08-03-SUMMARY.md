---
phase: 08-code-review-fixes
plan: 03
subsystem: gow-gzip
tags: [bug-fix, gnu-compatibility, gzip, stdin, suffix-rejection]
requirements: [FIX-05]

dependency_graph:
  requires: []
  provides: [WR-05-fix, IN-01-fix]
  affects: [crates/gow-gzip/src/lib.rs, crates/gow-gzip/tests/gzip_tests.rs]

tech_stack:
  added: []
  patterns:
    - "GNU suffix rejection: print error + set exit_code=1 + continue (not create spurious output file)"
    - "Simplified stdin error path: if-let-Err pattern, no dead result binding"

key_files:
  modified:
    - crates/gow-gzip/src/lib.rs
    - crates/gow-gzip/tests/gzip_tests.rs

decisions:
  - "WR-05: Reject files without .gz suffix in decompress mode (GNU-compatible behavior) — do not create .out files"
  - "IN-01: Remove dead result binding and misleading 'not in gzip format' message from stdin decompress path"

metrics:
  duration: "108s"
  completed: "2026-04-28T23:51:32Z"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 08 Plan 03: gow-gzip WR-05 and IN-01 Fixes Summary

**One-liner:** GNU-compatible suffix rejection for gzip -d (WR-05) plus simplified stdin decompress error path removing dead branch and hardcoded misleading message (IN-01).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix WR-05 (suffix rejection) and IN-01 (stdin dead code) in lib.rs | 1c0864b | crates/gow-gzip/src/lib.rs |
| 2 | Add WR-05 and IN-01 tests to gzip_tests.rs | f11b7d7 | crates/gow-gzip/tests/gzip_tests.rs |

## What Was Built

### WR-05: Suffix Rejection

In `Mode::Decompress` file loop inside `run()`, the `else` branch of the out_path derivation was changed from creating a spurious `.out` file to rejecting the input with a GNU-compatible error message:

**Before:**
```rust
} else {
    // GNU gzip appends ".out" if no .gz suffix; here we just try anyway
    format!("{converted}.out")
};
```

**After:**
```rust
} else {
    eprintln!("gzip: {converted}: unknown suffix -- ignored");
    exit_code = 1;
    continue;
};
```

This matches GNU gzip behavior: files without a `.gz` suffix passed to `gzip -d` are rejected immediately with exit code 1. No output file is created and the original file is not consumed.

### IN-01: Stdin Dead Code Simplification

The stdin block was simplified from a convoluted structure with a dead `result` binding and a misleading hardcoded error message to a clean symmetric pattern:

**Before (dead code, masked errors):**
```rust
let result = match mode {
    Mode::Compress => compress_stream(stdin.lock(), stdout.lock()),
    Mode::Decompress => {
        let res = decompress_stream(stdin.lock(), stdout.lock());
        if res.is_err() {
            eprintln!("gzip: stdin: not in gzip format");  // misleading
            return 1;
        }
        res  // dead: Err already returned above
    }
};
if let Err(e) = result {  // dead for Decompress arm
    eprintln!("gzip: stdin: {e}");
    exit_code = 1;
}
```

**After (clear, symmetric):**
```rust
match mode {
    Mode::Compress => {
        if let Err(e) = compress_stream(stdin.lock(), stdout.lock()) {
            eprintln!("gzip: stdin: {e}");
            return 1;
        }
        return 0;
    }
    Mode::Decompress => {
        if let Err(e) = decompress_stream(stdin.lock(), stdout.lock()) {
            eprintln!("gzip: stdin: {e}");
            return 1;
        }
        return 0;
    }
}
```

Both arms now emit the actual decoder error from flate2 instead of the hardcoded "not in gzip format" string.

## Tests Added

Three new integration tests added to `crates/gow-gzip/tests/gzip_tests.rs`:

1. `no_gz_suffix_rejected` — `gzip -d plainfile.txt` exits 1, stderr contains "unknown suffix"
2. `no_gz_suffix_does_not_create_out_file` — verifies no `.out` file created; original file preserved
3. `stdin_decompress_invalid_data_exits_1` — `gzip -d` on invalid stdin data exits 1, stderr references "stdin" or "gzip"

All 11 tests pass (8 pre-existing + 3 new).

## Verification Results

```
cargo build -p gow-gzip → Finished (exit 0)
cargo test -p gow-gzip  → 11 passed; 0 failed (exit 0)
grep '\.out"' lib.rs    → 0 lines (no .out fallback)
grep 'unknown suffix' lib.rs → 1 line
grep 'not in gzip format' lib.rs → 0 lines
```

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Both fixes reduce code paths (removing dead branch, removing .out creation) — attack surface is reduced, not expanded. This is consistent with threat register entries T-08-03-01 through T-08-03-04 (all accepted or mitigated as planned).

## Known Stubs

None.

## Self-Check: PASSED

- `crates/gow-gzip/src/lib.rs` — modified and committed (1c0864b)
- `crates/gow-gzip/tests/gzip_tests.rs` — modified and committed (f11b7d7)
- `.planning/phases/08-code-review-fixes/08-03-SUMMARY.md` — created
- Commit 1c0864b exists: `fix(08-03): WR-05 suffix rejection and IN-01 stdin dead code in gow-gzip`
- Commit f11b7d7 exists: `test(08-03): add WR-05 and IN-01 integration tests for gow-gzip`
