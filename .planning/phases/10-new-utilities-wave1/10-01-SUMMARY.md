---
phase: 10-new-utilities-wave1
plan: "01"
subsystem: workspace-scaffold
tags: [scaffold, workspace, crates, hashsum, multi-binary]
dependency_graph:
  requires: []
  provides:
    - crates/gow-seq
    - crates/gow-sleep
    - crates/gow-tac
    - crates/gow-nl
    - crates/gow-od
    - crates/gow-fold
    - crates/gow-expand-unexpand
    - crates/gow-du
    - crates/gow-df
    - crates/gow-hashsum
  affects:
    - Cargo.toml (workspace members + deps)
    - Cargo.lock (11 new packages)
tech_stack:
  added:
    - md-5 = "0.11"
    - sha1 = "0.11"
    - sha2 = "0.11"
    - digest = "0.11"
    - hex = "0.4"
  patterns:
    - multi-binary crate pattern (gow-gzip style: [[bin]] x N + [lib] uu_NAME)
    - Wave 0 stub uumain pattern (eprintln + exit 1)
    - hash linkage canary in gow-hashsum/src/lib.rs
key_files:
  created:
    - crates/gow-seq/Cargo.toml
    - crates/gow-seq/build.rs
    - crates/gow-seq/src/main.rs
    - crates/gow-seq/src/lib.rs
    - crates/gow-seq/tests/integration.rs
    - crates/gow-sleep/Cargo.toml
    - crates/gow-sleep/build.rs
    - crates/gow-sleep/src/main.rs
    - crates/gow-sleep/src/lib.rs
    - crates/gow-sleep/tests/integration.rs
    - crates/gow-tac/Cargo.toml
    - crates/gow-tac/build.rs
    - crates/gow-tac/src/main.rs
    - crates/gow-tac/src/lib.rs
    - crates/gow-tac/tests/integration.rs
    - crates/gow-nl/Cargo.toml
    - crates/gow-nl/build.rs
    - crates/gow-nl/src/main.rs
    - crates/gow-nl/src/lib.rs
    - crates/gow-nl/tests/integration.rs
    - crates/gow-od/Cargo.toml
    - crates/gow-od/build.rs
    - crates/gow-od/src/main.rs
    - crates/gow-od/src/lib.rs
    - crates/gow-od/tests/integration.rs
    - crates/gow-fold/Cargo.toml
    - crates/gow-fold/build.rs
    - crates/gow-fold/src/main.rs
    - crates/gow-fold/src/lib.rs
    - crates/gow-fold/tests/integration.rs
    - crates/gow-expand-unexpand/Cargo.toml
    - crates/gow-expand-unexpand/build.rs
    - crates/gow-expand-unexpand/src/expand.rs
    - crates/gow-expand-unexpand/src/unexpand.rs
    - crates/gow-expand-unexpand/src/lib.rs
    - crates/gow-expand-unexpand/tests/integration.rs
    - crates/gow-du/Cargo.toml
    - crates/gow-du/build.rs
    - crates/gow-du/src/main.rs
    - crates/gow-du/src/lib.rs
    - crates/gow-du/tests/integration.rs
    - crates/gow-df/Cargo.toml
    - crates/gow-df/build.rs
    - crates/gow-df/src/main.rs
    - crates/gow-df/src/lib.rs
    - crates/gow-df/tests/integration.rs
    - crates/gow-hashsum/Cargo.toml
    - crates/gow-hashsum/build.rs
    - crates/gow-hashsum/src/md5sum.rs
    - crates/gow-hashsum/src/sha1sum.rs
    - crates/gow-hashsum/src/sha256sum.rs
    - crates/gow-hashsum/src/lib.rs
    - crates/gow-hashsum/tests/integration.rs
  modified:
    - Cargo.toml (10 workspace members + 5 hash deps)
    - Cargo.lock (11 new packages locked)
decisions:
  - "Multi-binary crates (gow-expand-unexpand, gow-hashsum) follow gow-gzip pattern: each [[bin]] entry points to a thin main.rs that calls the shared uu_NAME::uumain()"
  - "Hash linkage canary in gow-hashsum/src/lib.rs uses #[allow(unused_imports)] + runtime hex::encode call to force MSVC linker to keep all five hash symbols"
  - "gow-df uses windows-sys workspace dep without feature annotation — workspace already has Win32_Storage_FileSystem enabled; inheritance carries the features"
metrics:
  duration: "9 minutes"
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 52
  files_modified: 2
---

# Phase 10 Plan 01: Scaffold 10 New Utility Crates Summary

**One-liner:** Workspace scaffold of 10 new GNU utility crates (seq/sleep/tac/nl/od/fold/expand/unexpand/du/df/md5sum/sha1sum/sha256sum) with stub uumain, Wave 0 placeholder tests, and MSVC hash linkage canary proving md-5/sha1/sha2/digest/hex compile and link.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 | Add 10 workspace members + 5 hash workspace deps to root Cargo.toml | 9eb5955 | Done |
| 2 | Scaffold all 10 crates (52 files) + Wave 0 tests; workspace build green | ba9468f | Done |

## Verification Results

- `cargo build --workspace` exits 0 — all 10 new crates compile as stubs
- `cargo build -p gow-hashsum` exits 0 — md-5/sha1/sha2/digest/hex MSVC link canary passes
- `cargo test --workspace --no-run` exits 0 — all Wave 0 test files compile
- `target/x86_64-pc-windows-msvc/debug/expand.exe` exists
- `target/x86_64-pc-windows-msvc/debug/unexpand.exe` exists
- `target/x86_64-pc-windows-msvc/debug/md5sum.exe` exists
- `target/x86_64-pc-windows-msvc/debug/sha1sum.exe` exists
- `target/x86_64-pc-windows-msvc/debug/sha256sum.exe` exists
- `cargo run -p gow-hashsum --bin md5sum -- --help` prints "hashsum: not implemented" and exits 1
- `cargo run -p gow-seq -- --help` prints "seq: not implemented" and exits 1

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

All 10 crates are intentional stubs. Each lib.rs has a `uumain` that prints
"<name>: not implemented" and exits 1. These stubs are the explicit goal of Plan 01;
real implementations are deferred to Wave 2+ plans per the phase structure.

The gow-hashsum lib.rs includes a non-stub linkage canary (actual hash crate imports
and `hex::encode` call) to prove the RustCrypto chain links successfully on MSVC.

## Self-Check: PASSED

- `crates/gow-seq/src/lib.rs` — FOUND
- `crates/gow-sleep/src/lib.rs` — FOUND
- `crates/gow-tac/src/lib.rs` — FOUND
- `crates/gow-nl/src/lib.rs` — FOUND
- `crates/gow-od/src/lib.rs` — FOUND
- `crates/gow-fold/src/lib.rs` — FOUND
- `crates/gow-expand-unexpand/src/lib.rs` — FOUND
- `crates/gow-du/src/lib.rs` — FOUND
- `crates/gow-df/src/lib.rs` — FOUND
- `crates/gow-hashsum/src/lib.rs` — FOUND
- Task 1 commit 9eb5955 — FOUND
- Task 2 commit ba9468f — FOUND
