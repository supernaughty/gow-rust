---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-04-20T13:56:00Z"
progress:
  total_phases: 6
  completed_phases: 0
  total_plans: 4
  completed_plans: 1
  percent: 4
---

# Project State: gow-rust

**Last updated:** 2026-04-20
**Session:** Executing Phase 1 Plan 01 (workspace scaffold + gow-core skeleton)

---

## Project Reference

**Core Value:** Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

**Current Focus:** Phase 1 — Foundation (gow-core shared library and Cargo workspace)

---

## Current Position

**Phase:** 1 (Foundation)
**Plan:** 01-01 complete; next plan is 01-02
**Status:** Executing

**Progress:**

```
Phase 1 [##        ] 25%  Foundation (1/4 plans complete)
Phase 2 [          ] 0%   Stateless Utilities
Phase 3 [          ] 0%   Filesystem Utilities
Phase 4 [          ] 0%   Text Processing
Phase 5 [          ] 0%   Search and Navigation
Phase 6 [          ] 0%   Archive, Compression, and Network
```

**Overall:** 0/6 phases complete (1/4 Phase 1 plans complete)

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases total | 6 |
| Phases complete | 0 |
| Requirements total | 59 |
| Requirements complete | 5 (FOUND-01, FOUND-02, WIN-01, WIN-02, WIN-03) |
| Plans created | 4 (Phase 1) |
| Plans complete | 1 (01-01) |

### Per-plan execution

| Phase-Plan | Duration | Tasks | Files | Completed |
|------------|----------|-------|-------|-----------|
| 01-01 | 6 min | 2 | 13 | 2026-04-20 |

---

## Accumulated Context

### Key Decisions Made

| Decision | Rationale |
|----------|-----------|
| Phase 1 must be gow-core first | Three critical pitfalls (clap exit codes, UTF-8 codepage, path conversion) are architectural and cannot be retrofitted across 20+ utilities |
| MSVC toolchain required | MinGW introduces a separate C runtime; MSVC is the only distribution-safe target for Windows |
| Individual binaries (not multicall) | Matches original GOW user expectation; revisit multicall in Phase 6 if Defender scan penalty proves unacceptable |
| MSI installer deferred to v2 | Depends on all binaries being stable; WiX multi-binary complexity is post-utility work |
| AWK moved to Phase 4 | AWK is a text processing tool; dependency on bstr/regex patterns proven in Phase 3 |
| tail -f in Phase 3 | Requires notify/ReadDirectoryChangesW; filesystem phase is the right boundary |
| curl in Phase 6 | tokio async runtime must be isolated from simpler coreutils to avoid compile-time bleed |
| gow-core stays lib-only in Plan 01 | embed-manifest's `cargo:rustc-link-arg-bins` directive is rejected by cargo 1.95 on lib-only packages; build.rs now detects bin targets before invoking embed_manifest so the same script works verbatim when copied into Phase 2+ utility bin crates (Plan 01-01) |
| embed-manifest API: Setting::Enabled, not LongPathAware::Yes | embed-manifest 1.5.0 uses a shared `Setting { Enabled, Disabled }` enum for boolean-style flags. Plan/research documents referenced a non-existent `LongPathAware::Yes`; corrected against the crate source (Plan 01-01) |
| assert_cmd 2.x has no `cargo` feature flag | `features = ["cargo"]` in the workspace dep caused resolution failure. Functionality exposed unconditionally in 2.x; declared plainly as `assert_cmd = "2"` (Plan 01-01) |
| Cargo.lock committed at repo root | Workspace produces binaries (FOUND-01, D-14); lockfile pins the exact 66-package dependency graph for reproducible CI (Plan 01-01) |

### Architecture Notes

- Cargo workspace with `gow-core` shared lib + one `uu_name` lib crate per utility + thin `name.exe` binary wrappers
- gow-core centralizes: UTF-8 init (SetConsoleOutputCP 65001 + app manifest), path normalization (context-aware, not regex), ANSI/VT100 init, shared error types, GNU arg parsing abstraction over lexopt/clap
- No utility crate knows about Win32 directly; all platform specifics in gow-core
- test stack: assert_cmd + snapbox for GNU compatibility snapshot tests

### Critical Pitfalls to Watch

1. clap exits code 2 on bad args; GNU requires 1 — must be fixed in gow-core::args before any utility is written
2. Path conversion `/c/Users` → `C:\Users` must be context-aware or it corrupts `-c` flag arguments
3. tail -f must watch the parent directory (not the file) via ReadDirectoryChangesW with filename filtering
4. sed -i must use MoveFileExW(MOVEFILE_REPLACE_EXISTING) for atomic temp-file swap under Windows file locking
5. find -exec arguments must be passed as argv array, never through a shell string

### Blockers

None currently.

### Research Flags (addressed during planning)

- Phase 5 (find/xargs): Windows process creation and argument quoting needs a spike during plan-phase
- Phase 6 (curl): TLS/SChannel on corporate Windows (proxies, certificate stores) needs native-tls + reqwest Windows validation

---

## Session Continuity

### What Was Done This Session

- Executed Phase 1 Plan 01 (workspace scaffold + gow-core skeleton)
  - Task 1 (`974a7fe`): workspace Cargo.toml, .cargo/config.toml, resolver 3, edition 2024, +crt-static
  - Task 2 (`c15706b`): gow-core crate manifest, bin-gated build.rs, six module stubs, init() smoke test
- 5 auto-fixed deviations documented in SUMMARY:
  - Rule 1 (bug) × 2: assert_cmd feature flag, embed-manifest Setting enum
  - Rule 3 (blocking) × 1: bin-target-aware embed_manifest() gating
  - Rule 2 (missing critical) × 2: init smoke test, .gitignore
- `cargo build --workspace` and `cargo test -p gow-core` both green; clippy -D warnings clean
- 66-package Cargo.lock committed; 13 files created; 1 file modified (initial Cargo.toml fix)

### What To Do Next

Run the executor on Plan 01-02 (encoding, args, color modules with unit tests). Dependencies (clap, thiserror, termcolor, windows-sys) are already in scope via workspace inheritance; module files exist at `crates/gow-core/src/{encoding,args,color}.rs` with no-op stubs ready to be filled in.

---

*State initialized: 2026-04-20*
*Plan 01-01 completed: 2026-04-20*
