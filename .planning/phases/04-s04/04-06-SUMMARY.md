---
phase: "04-s04"
plan: "06"
subsystem: "text-processing"
tags: ["diff", "patch", "unified-diff", "similar", "diffy", "tdd"]
dependency_graph:
  requires: ["04-01"]
  provides: ["gow-diff", "gow-patch"]
  affects: ["Cargo.toml", "crates/gow-diff", "crates/gow-patch"]
tech_stack:
  added: ["similar = 2.7.0", "diffy = 0.4.2"]
  patterns: ["TDD RED/GREEN", "atomic_rewrite", "diffy::apply", "TextDiff::from_lines"]
key_files:
  created:
    - crates/gow-diff/src/lib.rs
    - crates/gow-diff/tests/integration.rs
    - crates/gow-patch/src/lib.rs
    - crates/gow-patch/tests/integration.rs
  modified:
    - Cargo.toml
    - crates/gow-diff/Cargo.toml
    - crates/gow-patch/Cargo.toml
decisions:
  - "Used similar 2.7.0 (not 3.x) because workspace dep pinned to '2.6' — resolved to 2.7.0"
  - "UnifiedHunkHeader uses Display impl for @@ formatting; fields are private in similar 2.x"
  - "diffy::Patch has .reverse() method supporting -R without manual hunk swapping"
  - "strip_path implemented manually via char-by-char scan to handle both / and \\ separators"
  - "patch uses atomic_rewrite for all file writes; --dry-run validates only without writing"
metrics:
  duration: "~20 minutes"
  completed: "2026-04-25T11:10:44Z"
  tasks_completed: 2
  files_created: 4
  files_modified: 3
---

# Phase 04 Plan 06: Diff + Patch Utilities Summary

**One-liner:** GNU-compatible diff (unified format via similar crate) and patch (atomic apply via diffy) with full TDD test coverage.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 (RED) | Failing integration tests for gow-diff | d2bd463 | tests/integration.rs, Cargo.toml |
| 1 (GREEN) | Implement gow-diff with similar crate | 8bb6ecf | src/lib.rs |
| 2 (RED) | Failing integration tests for gow-patch | a1ae9a9 | tests/integration.rs, Cargo.toml |
| 2 (GREEN) | Implement gow-patch with diffy crate | b9d7293 | src/lib.rs |

## What Was Built

### gow-diff
- Unified diff file comparison using `similar::TextDiff::from_lines`
- Flags: `-u`/`-U N` (context lines, default 3), `-r` (recursive directory), `-N` (absent-as-empty)
- Exit codes: 0=identical, 1=differences, 2=error (GNU-compatible)
- Unified format headers: `--- path\ttimestamp`, `+++ path\ttimestamp`, `@@ -L,S +L,S @@`
- Recursive comparison with "Only in dir: file" reporting for unmatched files
- Gregorian timestamp formatting using pure arithmetic (no external time crate needed)

### gow-patch
- Unified diff application using `diffy::apply` and `diffy::Patch::from_str`
- Flags: `-p N` (strip N path components, default 1), `-R` (reverse via `Patch::reverse()`), `--dry-run`, `-i FILE`
- Atomic file writes via `gow_core::fs::atomic_rewrite` — partial patches never written
- Path traversal protection: `strip_path` normalizes components; operates relative to CWD
- Exit codes: 0=success, 1=apply failure, 2=parse error

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] similar crate API mismatch — UnifiedHunkHeader fields are private**
- **Found during:** Task 1 GREEN implementation
- **Issue:** Plan showed `header.old_start()` / `header.old_len()` etc. but these methods don't exist in similar 2.x; `UnifiedHunkHeader` only has `Display`
- **Fix:** Used `println!("{}", hunk.header())` to leverage the built-in `@@ -L,S +L,S @@` Display format
- **Files modified:** crates/gow-diff/src/lib.rs
- **Commit:** 8bb6ecf

**2. [Rule 1 - Bug] Lifetime error with unified_diff builder chaining**
- **Found during:** Task 1 GREEN implementation
- **Issue:** `diff.unified_diff().context_radius(context)` created a temporary dropped while borrowed
- **Fix:** Separated into `let mut unified_builder = diff.unified_diff(); let unified = unified_builder.context_radius(context);`
- **Files modified:** crates/gow-diff/src/lib.rs
- **Commit:** 8bb6ecf

## TDD Gate Compliance

Both tasks followed the full RED/GREEN cycle:
1. RED commit with failing tests (d2bd463, a1ae9a9)
2. GREEN commit with passing implementation (8bb6ecf, b9d7293)

No REFACTOR pass was needed — code is clean as written.

## Threat Surface Review

The following threat mitigations from the plan's threat model were implemented:

| Threat ID | Mitigation Applied |
|-----------|-------------------|
| T-04-06-01 | strip_path normalizes component count; resulting path is relative, joined to CWD by std::fs operations |
| T-04-06-03 | atomic_rewrite ensures no partial writes; .rej files not written (patches either succeed or fail atomically) |
| T-04-06-04 | WalkDir respects filesystem permissions; unreadable files produce stderr warning |

T-04-06-02 (large diff) and T-04-06-05 (command execution) accepted per plan — no mitigation code needed.

## Known Stubs

None — both utilities are fully wired. `diff` reads actual file content and produces real unified diffs; `patch` reads real patch content and applies real changes.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| crates/gow-diff/src/lib.rs exists (267 lines, >150) | FOUND |
| crates/gow-diff/tests/integration.rs exists | FOUND |
| crates/gow-patch/src/lib.rs exists (167 lines, >150) | FOUND |
| crates/gow-patch/tests/integration.rs exists | FOUND |
| Commit d2bd463 (diff RED tests) | FOUND |
| Commit 8bb6ecf (diff GREEN impl) | FOUND |
| Commit a1ae9a9 (patch RED tests) | FOUND |
| Commit b9d7293 (patch GREEN impl) | FOUND |
| TextDiff::from_lines in diff lib.rs | FOUND |
| fn diff_files in diff lib.rs | FOUND |
| diffy::apply in patch lib.rs | FOUND |
| atomic_rewrite in patch lib.rs | FOUND |
| fn strip_path in patch lib.rs | FOUND |
| cargo test -p gow-diff -p gow-patch (11 tests) | PASSED |
