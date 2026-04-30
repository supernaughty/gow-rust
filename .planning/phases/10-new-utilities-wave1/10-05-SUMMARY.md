---
phase: 10-new-utilities-wave1
plan: "05"
subsystem: disk-utilities
tags: [du, df, walkdir, windows-sys, disk-usage, disk-free, human-readable]
dependency_graph:
  requires:
    - 10-01 (crate scaffold)
  provides:
    - crates/gow-du (U-08 complete)
    - crates/gow-df (U-09 complete)
  affects:
    - crates/gow-du/src/lib.rs
    - crates/gow-du/tests/integration.rs
    - crates/gow-df/src/lib.rs
    - crates/gow-df/tests/integration.rs
tech_stack:
  added: []
  patterns:
    - WalkDir.follow_links(false) for symlink-safe directory traversal
    - GetLogicalDriveStringsW + GetDiskFreeSpaceExW for Windows drive enumeration
    - human_readable() binary SI units (1K=1024) shared pattern
    - Explicit unsafe block inside safe wrapper function (Rust 2024 edition)
key_files:
  created: []
  modified:
    - crates/gow-du/src/lib.rs
    - crates/gow-du/tests/integration.rs
    - crates/gow-df/src/lib.rs
    - crates/gow-df/tests/integration.rs
decisions:
  - "get_disk_free() made a safe fn with inner unsafe block per Rust 2024 edition requirement — unsafe_op_in_unsafe_fn lint caught at compile time"
  - "du non-summarize mode uses O(N^2) per-directory recursion (dir_usage_recursive called once per subdir) — acceptable for Phase 10; linear post-order accumulator is a stretch goal"
  - "WalkDir.max_depth(wd_max_depth) passed directly: GNU du --max-depth=0 shows only the root, depth 0 = only root entry in WalkDir"
metrics:
  duration: "5 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 4
---

# Phase 10 Plan 05: gow-du + gow-df Implementation Summary

**One-liner:** du (U-08) implemented with WalkDir.follow_links(false) + 1K-block/human-readable output; df (U-09) implemented with GetLogicalDriveStringsW + GetDiskFreeSpaceExW, silently skipping unresponsive drives.

## Tasks Completed

| Task | Description | Commits | Files |
|------|-------------|---------|-------|
| 1 (TDD) | gow-du: walkdir disk usage, -s/-h/-a/-b/-d flags | 761301d (RED), 4e8b44d (GREEN) | lib.rs, tests/integration.rs |
| 2 (TDD) | gow-df: Windows drive enumeration + disk free | 8bf22ab (RED), 5b291a9 (GREEN) | lib.rs, tests/integration.rs |

## Verification Results

- `cargo test -p gow-du`: 6/6 integration tests pass
- `cargo test -p gow-df`: 5/5 integration tests pass
- `cargo test -p gow-du -p gow-df`: 11/11 total tests pass
- `cargo build --workspace`: exits 0 — no regressions
- `cargo run -p gow-du -- -sh .`: outputs "368M\t." (human-readable, tab-separated)
- `cargo run -p gow-df`: outputs GNU-format header + one row per mounted drive
- `cargo run -p gow-df -- -h`: outputs "Size" header + human-readable columns

## Acceptance Criteria

- [x] `grep "follow_links(false)" crates/gow-du/src/lib.rs` — match found
- [x] `grep "WalkDir" crates/gow-du/src/lib.rs` — match found
- [x] `grep "human_readable" crates/gow-du/src/lib.rs` — match found
- [x] `grep "not implemented" crates/gow-du/src/lib.rs` — no match (stub removed)
- [x] `grep "GetDiskFreeSpaceExW" crates/gow-df/src/lib.rs` — match found
- [x] `grep "GetLogicalDriveStringsW" crates/gow-df/src/lib.rs` — match found
- [x] `grep "Filesystem" crates/gow-df/src/lib.rs` — match found
- [x] `grep "1K-blocks" crates/gow-df/src/lib.rs` — match found
- [x] `grep "not implemented" crates/gow-df/src/lib.rs` — no match (stub removed)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unsafe_op_in_unsafe_fn warning in gow-df**
- **Found during:** Task 2 GREEN phase, cargo test output
- **Issue:** Rust 2024 edition requires explicit `unsafe {}` blocks inside `unsafe fn` bodies; the warning promoted to error-level lint
- **Fix:** Converted `unsafe fn get_disk_free()` to a safe `fn` with an inner `unsafe { GetDiskFreeSpaceExW(...) }` block; removed `unsafe` call site in `run()`
- **Files modified:** `crates/gow-df/src/lib.rs`
- **Commit:** 5b291a9 (included in GREEN commit)

## TDD Gate Compliance

Both tasks followed strict RED/GREEN sequence:

| Task | RED commit | GREEN commit |
|------|-----------|-------------|
| 1 (du) | 761301d | 4e8b44d |
| 2 (df) | 8bf22ab | 5b291a9 |

No REFACTOR commits needed — code was clean after GREEN.

## Threat Mitigations Applied

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-10-05-01: cyclic symlinks in du | `WalkDir::new(path).follow_links(false)` | grep confirms |
| T-10-05-02: unsafe Win32 calls in df | Minimal local-pointer unsafe block; safe fn wrapper | Code review |
| T-10-05-03: unresponsive drives in df | `None` return from `get_disk_free` → `continue` (silent skip) | Integration test passes on multi-drive system |

## Known Stubs

None — both crates are fully implemented.

## Self-Check: PASSED

- `crates/gow-du/src/lib.rs` — FOUND
- `crates/gow-du/tests/integration.rs` — FOUND
- `crates/gow-df/src/lib.rs` — FOUND
- `crates/gow-df/tests/integration.rs` — FOUND
- Task 1 RED commit 761301d — FOUND
- Task 1 GREEN commit 4e8b44d — FOUND
- Task 2 RED commit 8bf22ab — FOUND
- Task 2 GREEN commit 5b291a9 — FOUND
