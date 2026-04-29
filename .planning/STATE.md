---
gsd_state_version: 1.0
milestone: v0.2.0
milestone_name: "- [x] **Phase 07: release-and-ci** — Release & CI/CD *"
status: completed
stopped_at: Completed 11-06-PLAN.md — whoami + uname implemented, workspace test gate green
last_updated: "2026-04-29T21:25:03.885Z"
last_activity: 2026-04-29 -- Phase 11 plan 01 complete (10 crates scaffolded)
progress:
  total_phases: 7
  completed_phases: 7
  total_plans: 31
  completed_plans: 31
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-29)

**Current focus:** Phase 11 — new-utilities-wave2 (Plan 01 complete, executing plan 02)

## Current Position

Phase: 11 (new-utilities-wave2) — EXECUTING
Plan: 11-02 (next: whoami + uname implementation)
Status: Plan 01 complete — 10 crates scaffolded, workspace green
Last activity: 2026-04-29 -- Phase 11 plan 01 complete (10 crates scaffolded)

Progress: [██████████] 100%

## Accumulated Context

### Decisions

Migrated from GSD-2. Review PROJECT.md for key decisions.

- 04-06: Used similar 2.7 (not 3.x); UnifiedHunkHeader uses Display impl for @@ formatting
- 04-06: diffy::Patch::reverse() handles -R without manual hunk manipulation
- 04-06: strip_path implemented via char-by-char scan to handle both / and \\ separators
- 04-06: patch uses atomic_rewrite; --dry-run validates only without writing
- 04-07: Built POSIX AWK from scratch (frawk=binary, rawk=not production-ready); regex+bstr approach
- 04-07: print > file redirect disabled (T-04-07-03 security), system() disabled (T-04-07-04)
- 04-07: -v variable names validated alphanumeric+underscore (T-04-07-06)
- 05-01: crossterm = 0.29 added as workspace dep for gow-less (D-10); globset = 0.4 for gow-find (D-02)
- 05-02: normalize_find_args() rewrites single-dash GNU flags to double-dash before clap — parse_gnu() does not handle single-dash long flags natively
- 05-02: allow_hyphen_values=true on -mtime/-atime/-ctime/-size so '-N' values accepted by clap
- 05-03: exec_batch/exec_with_replacement return Option<i32> to distinguish signal-killed (None) from exit code
- 05-04: LineIndex uses Vec<u64> byte offsets, forward scan only, seek for random access (D-09)
- 05-04: Stdin buffered to NamedTempFile for seekability in less (matches GNU less behavior)
- 05-04: tempfile added to [dependencies] (not just dev-deps) for runtime stdin buffering
- 07-02: branches: ["**"] on push EXCLUDES tag pushes — prevents duplicate CI runs when v* tags are pushed
- 07-02: dtolnay/rust-toolchain@stable used (not deprecated actions-rs/toolchain)
- 07-02: Swatinem/rust-cache@v2 key x86_64-pc-windows-msvc differentiates from release workflow cache keys
- softprops/action-gh-release@v2 chosen over v3 — v3 requires Node 24, not available on windows-latest
- ilammy/msvc-dev-cmd@v1 placed after rust-toolchain and before rust-cache for liblzma-sys 32-bit C compilation
- download-extras.ps1 added to both CI build jobs — runners have no extras/bin/ pre-populated
- 11-01: gow-test binary named 'test'; [.bat shim dispatches to test.exe with --_bracket_ sentinel for bracket alias
- 11-01: No per-crate features= on windows-sys — workspace inheritance handles Win32_System_WindowsProgramming/SystemInformation/Wdk_System_SystemServices
- Used Vec<String> for uname parts accumulator to avoid borrow-checker lifetime issues with format! string temporaries
- GetVersionExW appears only in doc comments in uname; RtlGetVersion used exclusively — verified by smoke test showing 10.0.26200 not 6.2

### Known Issues (from code review 05-REVIEW.md)

- CR-01: find -exec exit code not propagated — run() always returns Ok(()); GNU find should exit 1 on exec failure
- CR-02: NamedTempFile deleted before LineIndex reads in less stdin path on Windows — use tempfile::tempfile() instead

### Known Issues (from code review 06-REVIEW.md — to be fixed in Phase 08)

- WR-01: tar uses BzDecoder instead of MultiBzDecoder — multi-stream .tar.bz2 truncates (FIX-01)
- WR-02: tar Cli::from_arg_matches().unwrap() panics instead of graceful error + exit 2 (FIX-02)
- WR-03: tar unpack_archive returns Ok(()) on extraction errors — exit code 0 on partial extract (FIX-03)
- WR-04: xz uses single-stream XzDecoder — concatenated .xz files silently truncate (FIX-04)
- WR-05: gzip appends .out for files without .gz suffix instead of rejecting (GNU incompatibility) (FIX-05)
- WR-06: curl -I -s prints headers in silent mode (should be suppressed) (FIX-06)
- WR-07: curl -o leaves partial file on I/O error (should remove on failure) (FIX-07)

### Blockers/Concerns

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260429-q01 | Multi-arch builds (x64/x86/ARM64) + WiX v3 MSI installer infrastructure | 2026-04-29 | e1d6fb1 | [260429-q01-multi-arch-msi](.planning/quick/260429-q01-multi-arch-msi/) |

## Session Continuity

Last session: 2026-04-29T21:25:03.879Z
Stopped at: Completed 11-06-PLAN.md — whoami + uname implemented, workspace test gate green
Resume file: None

**Next Plan:** 11-02 — whoami + uname implementation
