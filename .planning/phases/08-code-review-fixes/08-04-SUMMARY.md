---
phase: 08-code-review-fixes
plan: "04"
subsystem: gow-curl
tags: [bugfix, silent-mode, file-cleanup, gnu-compatibility, wr-06, wr-07]
requirements: [FIX-06, FIX-07]
dependency_graph:
  requires: []
  provides: [WR-06-silent-header-guard, WR-07-partial-file-cleanup]
  affects: [crates/gow-curl/src/lib.rs, crates/gow-curl/tests/curl_tests.rs]
tech_stack:
  added: []
  patterns:
    - "Wrapping io::copy in if let Err(e) for partial file cleanup (matches gzip/bzip2/xz pattern)"
    - "Extending !cli.silent guard block to cover all output in HEAD branch"
key_files:
  created: []
  modified:
    - crates/gow-curl/src/lib.rs
    - crates/gow-curl/tests/curl_tests.rs
decisions:
  - "WR-06: move header for loop inside existing !cli.silent block rather than adding a second guard — single block makes the invariant clear"
  - "WR-07: use let _ = fs::remove_file(...) to swallow remove_file errors and preserve the original io::copy error — matches gzip/bzip2/xz pattern in the codebase"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-28T23:51:53Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 08 Plan 04: WR-06 Silent Header Guard and WR-07 Partial File Cleanup Summary

**One-liner:** Fixed curl -s -I silent mode by moving the header loop inside the !cli.silent guard (WR-06), and added partial file removal on io::copy failure to prevent truncated output files (WR-07).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix WR-06 and WR-07 in lib.rs | 9975da1 | crates/gow-curl/src/lib.rs |
| 2 | Add WR-06 and WR-07 tests | 7706599 | crates/gow-curl/tests/curl_tests.rs |

## What Was Done

### Task 1: Fix WR-06 (Silent Header Guard)

The header-printing `for (name, value) in response.headers()` loop was outside the `if !cli.silent` block, causing `curl -s -I <url>` to print headers to stdout even in silent mode. The loop was moved inside the guard so the entire HEAD output (status line + headers) is suppressed when `-s` is active.

**Before:**
```rust
if !cli.silent {
    println!("HTTP/1.1 {}", status);
}
for (name, value) in response.headers() {   // NOT guarded
    println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
}
```

**After:**
```rust
if !cli.silent {
    println!("HTTP/1.1 {}", status);
    for (name, value) in response.headers() {   // now guarded
        println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
}
```

### Task 1: Fix WR-07 (Partial File Cleanup)

The bare `io::copy(&mut response, &mut file)?;` would propagate the error up via `?` but leave a partially-written file at `cli.output` on disk. This was replaced with an `if let Err(e)` arm that calls `fs::remove_file(output_path)` before returning the error, matching the cleanup pattern used by gzip, bzip2, and xz utilities.

The `use std::fs::File` import was updated to `use std::fs::{self, File}` to enable `fs::remove_file`.

### Task 2: Tests Added

Four new test functions were appended to `curl_tests.rs`:

- `silent_head_suppresses_all_output` — `#[ignore]` network test; verifies `curl -s -I` stdout is empty
- `non_silent_head_prints_headers` — `#[ignore]` network test; verifies `curl -I` without `-s` still prints headers
- `output_file_not_created_on_invalid_path` — offline test; verifies no partial file left when path is invalid
- `output_flag_accepted` — offline test; verifies `-o <file>` is accepted by clap argument parser

## Verification Results

```
cargo build -p gow-curl  → Finished dev profile — 0 errors
cargo test  -p gow-curl  → 4 passed, 0 failed, 6 ignored
```

- `grep "remove_file" crates/gow-curl/src/lib.rs` → 1 line: `let _ = fs::remove_file(output_path);`
- `grep "io::copy.*?" crates/gow-curl/src/lib.rs` → 0 lines (bare `?` removed)
- Header for loop confirmed inside `if !cli.silent { ... }` block
- `use std::fs::{self, File};` confirmed in imports

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Both fixes reduce code surface (header loop moves inside guard; io::copy error path gains cleanup — no new paths opened).

## Self-Check: PASSED

- [x] `crates/gow-curl/src/lib.rs` exists and contains `remove_file`
- [x] `crates/gow-curl/tests/curl_tests.rs` exists and contains `silent_head_suppresses_all_output`
- [x] Commit `9975da1` exists (Task 1)
- [x] Commit `7706599` exists (Task 2)
