---
phase: "05-search-and-navigation"
plan: "01"
subsystem: "search-and-navigation"
tags: [scaffold, workspace, find, xargs, less, crossterm, globset]
dependency_graph:
  requires: []
  provides: [gow-find-scaffold, gow-xargs-scaffold, gow-less-scaffold, crossterm-workspace-dep, globset-workspace-dep]
  affects: [Cargo.toml, Cargo.lock]
tech_stack:
  added: [crossterm@0.29, globset@0.4]
  patterns: [stub-uumain, embed-manifest, workspace-dep-declaration]
key_files:
  created:
    - crates/gow-find/Cargo.toml
    - crates/gow-find/build.rs
    - crates/gow-find/src/main.rs
    - crates/gow-find/src/lib.rs
    - crates/gow-xargs/Cargo.toml
    - crates/gow-xargs/build.rs
    - crates/gow-xargs/src/main.rs
    - crates/gow-xargs/src/lib.rs
    - crates/gow-less/Cargo.toml
    - crates/gow-less/build.rs
    - crates/gow-less/src/main.rs
    - crates/gow-less/src/lib.rs
  modified:
    - Cargo.toml
    - Cargo.lock
decisions:
  - "Used globset = 0.4 as workspace dep for gow-find (D-02); gow-xargs does NOT depend on globset"
  - "Used crossterm = 0.29 as workspace dep for gow-less (D-10); gow-find and gow-xargs do NOT depend on crossterm"
  - "windows-sys included in both gow-find and gow-xargs for future _setmode binary mode usage (_print0 / -0 flags)"
  - "gow-less includes terminal_size and regex as workspace deps for viewport sizing and / search"
metrics:
  duration_seconds: 140
  completed_date: "2026-04-28"
  tasks_completed: 2
  tasks_total: 2
  files_created: 12
  files_modified: 2
---

# Phase 05 Plan 01: Scaffold gow-find, gow-xargs, gow-less — Summary

**One-liner:** Three new workspace crates scaffolded with stub `uumain` implementations; crossterm 0.29 and globset 0.4 added as workspace-level dependencies; full workspace builds cleanly.

## What Was Built

This plan created the structural foundation for Phase 05's three utilities:

- **gow-find** (12 source files): GNU `find` scaffold with walkdir + globset + windows-sys deps
- **gow-xargs** (4 source files): GNU `xargs` scaffold with bstr + windows-sys deps, no walkdir/regex/globset
- **gow-less** (4 source files): GNU `less` scaffold with crossterm + terminal_size + regex deps

Each crate follows the established pattern: 3-line `main.rs` delegating to `uu_<name>::uumain`, stub `lib.rs` calling `gow_core::init()` then printing `<name>: not implemented` with exit 1, `build.rs` with Windows manifest embedding (UTF-8 + long path aware), and an empty `tests/` directory ready for plan 05-02/03/04.

Two new workspace-level dependencies were declared in the root `Cargo.toml`:
- `crossterm = "0.29"` — interactive terminal pager raw mode for `gow-less`
- `globset = "0.4"` — compiled glob pattern matching for `gow-find -name`

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build --workspace` | Exit 0 — all crates compile |
| `cargo metadata --no-deps` | Lists gow-find, gow-xargs, gow-less |
| `find --any-flag` | Prints `find: not implemented`, exit 1 |
| `xargs --x` | Prints `xargs: not implemented`, exit 1 |
| `less --x` | Prints `less: not implemented`, exit 1 |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

All three `lib.rs` files are intentional stubs. They will be replaced by:
- `crates/gow-find/src/lib.rs` → Plan 05-02 (full find implementation)
- `crates/gow-xargs/src/lib.rs` → Plan 05-03 (full xargs implementation)
- `crates/gow-less/src/lib.rs` → Plan 05-04 (full less pager implementation)

The stubs correctly satisfy the Phase 05 scaffolding goal: each prints an unambiguous "not implemented" message rather than silently failing or returning success.

## Threat Surface Scan

No new security-relevant surface introduced. This plan creates skeleton stubs only; the real attack surfaces (path traversal via -exec, stdin binary mode, raw terminal mode) are introduced in plans 05-02/03/04. The only manifest change is Cargo.toml workspace registration.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 — Workspace Cargo.toml | `1b77879` | feat(05-01): add gow-find, gow-xargs, gow-less workspace members + crossterm/globset deps |
| Task 2 — Scaffold three crates | `1cf466b` | feat(05-01): scaffold gow-find, gow-xargs, gow-less stub crates |

## Self-Check: PASSED

- [x] crates/gow-find/Cargo.toml exists
- [x] crates/gow-xargs/Cargo.toml exists
- [x] crates/gow-less/Cargo.toml exists
- [x] Commit 1b77879 exists
- [x] Commit 1cf466b exists
- [x] cargo build --workspace exit 0 verified
