---
phase: 11-new-utilities-wave2
plan: "01"
subsystem: workspace-scaffold
tags: [scaffold, workspace, phase11, whoami, uname, paste, join, split, printf, expr, test, fmt, unlink]
dependency_graph:
  requires: []
  provides:
    - crates/gow-whoami (stub binary + lib)
    - crates/gow-uname (stub binary + lib)
    - crates/gow-paste (stub binary + lib)
    - crates/gow-join (stub binary + lib)
    - crates/gow-split (stub binary + lib)
    - crates/gow-printf (stub binary + lib)
    - crates/gow-expr (stub binary + lib)
    - crates/gow-test (stub binary + lib)
    - crates/gow-fmt (stub binary + lib)
    - crates/gow-unlink (stub binary + lib)
    - extras/bin/[.bat (bracket alias shim)
  affects:
    - Cargo.toml (workspace members + windows-sys features)
    - Cargo.lock (updated with new dependency resolutions)
tech_stack:
  added:
    - embed-manifest = "1.5" (build-dep for each new crate, already in use by prior crates)
  patterns:
    - stub uumain pattern (eprintln + exit 1)
    - Wave 0 integration test placeholder (scaffold_compiles)
    - workspace dependency inheritance (no per-crate features= on windows-sys)
key_files:
  created:
    - Cargo.toml (modified — 10 new workspace members + 3 new windows-sys features)
    - extras/bin/[.bat (bracket alias shim)
    - crates/gow-whoami/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-uname/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-paste/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-join/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-split/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-printf/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-expr/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-test/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-fmt/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
    - crates/gow-unlink/{Cargo.toml,build.rs,src/main.rs,src/lib.rs,tests/integration.rs}
  modified:
    - Cargo.toml
    - Cargo.lock
decisions:
  - "gow-test binary named 'test' — single [[bin]] entry; [.bat shim dispatches to test.exe with --_bracket_ sentinel for bracket alias"
  - "No per-crate features= on windows-sys — workspace inheritance handles Win32_System_WindowsProgramming, Win32_System_SystemInformation, Wdk_System_SystemServices"
  - "bstr dep added to paste/join/split/fmt; regex dep to expr; windows-sys dep to whoami/uname; printf/test/unlink have only core deps"
metrics:
  duration_seconds: 225
  completed_date: "2026-04-29"
  tasks_completed: 2
  tasks_total: 2
  files_created: 52
  files_modified: 2
---

# Phase 11 Plan 01: Workspace Scaffold — 10 New Phase 11 Crates

**One-liner:** Scaffold 10 stub crates (whoami/uname/paste/join/split/printf/expr/test/fmt/unlink) with correct per-crate deps, expand windows-sys workspace features for Win32 system info APIs, and create the [.bat bracket alias shim.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Add 10 workspace members, expand windows-sys features, create [.bat | 21e82be | Cargo.toml, extras/bin/[.bat |
| 2 | Scaffold all 10 crates with stub uumain + Wave 0 test files | 99707ae | 50 new crate files + Cargo.lock |

## Verification Results

- `cargo metadata --no-deps` exits 0: PASS
- `cargo build --workspace` exits 0: PASS (32.71s)
- `cargo test --workspace --no-run` exits 0: PASS
- `whoami.exe` prints "whoami: not implemented", exits 1: PASS
- `test.exe` prints "test: not implemented", exits 1: PASS
- `Win32_System_WindowsProgramming` in Cargo.toml: PASS
- `Win32_System_SystemInformation` in Cargo.toml: PASS
- `Wdk_System_SystemServices` in Cargo.toml: PASS
- `extras/bin/[.bat` contains `--_bracket_`: PASS

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

All 10 new crates are intentional stubs (Wave 0). Each lib.rs prints "<name>: not implemented" and returns exit code 1. These stubs are expected and will be replaced by full implementations in Wave 2 plans (11-02 through 11-06).

| Stub | File | Reason |
|------|------|--------|
| whoami stub | crates/gow-whoami/src/lib.rs | Wave 0 scaffold; implemented in plan 11-02 |
| uname stub | crates/gow-uname/src/lib.rs | Wave 0 scaffold; implemented in plan 11-02 |
| paste stub | crates/gow-paste/src/lib.rs | Wave 0 scaffold; implemented in plan 11-03 |
| join stub | crates/gow-join/src/lib.rs | Wave 0 scaffold; implemented in plan 11-03 |
| split stub | crates/gow-split/src/lib.rs | Wave 0 scaffold; implemented in plan 11-04 |
| printf stub | crates/gow-printf/src/lib.rs | Wave 0 scaffold; implemented in plan 11-04 |
| expr stub | crates/gow-expr/src/lib.rs | Wave 0 scaffold; implemented in plan 11-05 |
| test stub | crates/gow-test/src/lib.rs | Wave 0 scaffold; implemented in plan 11-05 |
| fmt stub | crates/gow-fmt/src/lib.rs | Wave 0 scaffold; implemented in plan 11-06 |
| unlink stub | crates/gow-unlink/src/lib.rs | Wave 0 scaffold; implemented in plan 11-06 |

## Self-Check: PASSED

- [x] extras/bin/[.bat exists and contains --_bracket_
- [x] All 10 crate Cargo.toml files created
- [x] Commits 21e82be and 99707ae present in git log
- [x] cargo build --workspace exits 0
- [x] cargo test --workspace --no-run exits 0
