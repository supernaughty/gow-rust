---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-04-21T00:21:07Z"
progress:
  total_phases: 6
  completed_phases: 1
  total_plans: 15
  completed_plans: 5
  percent: 20
---

# Project State: gow-rust

**Last updated:** 2026-04-21
**Session:** Phase 2 Plan 01 (workspace prep + 14 stub crates) complete — ready for Phase 2 Wave 2 plans

---

## Project Reference

**Core Value:** Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

**Current Focus:** Phase 2 (Stateless Utilities) — workspace + stubs scaffolded; Wave 2 utility plans (02-02..02-05) next

---

## Current Position

**Phase:** 2 (Stateless Utilities) — IN PROGRESS
**Plan:** 02-01 complete (workspace root + 14 stub crates). Next up: Wave 2 of Phase 2 (plans 02-02 through 02-05 per ROADMAP.md; all can be parallelized since 02-01 unblocked the workspace).
**Status:** In progress

**Progress:**

```
Phase 1 [##########] 100% Foundation ✓ (4/4 plans, 10/10 requirements verified)
Phase 2 [#         ] 9%   Stateless Utilities (1/11 plans)
Phase 3 [          ] 0%   Filesystem Utilities
Phase 4 [          ] 0%   Text Processing
Phase 5 [          ] 0%   Search and Navigation
Phase 6 [          ] 0%   Archive, Compression, and Network
```

**Overall:** 1/6 phases complete (Phase 2: 1/11 plans complete)

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases total | 6 |
| Phases complete | 1 |
| Requirements total | 59 |
| Requirements complete | 10 (all Phase 1: FOUND-01..07, WIN-01..03) |
| Plans created | 15 (4 Phase 1 + 11 Phase 2) |
| Plans complete | 5 (01-01, 01-02, 01-03, 01-04, 02-01) |

### Per-plan execution

| Phase-Plan | Duration | Tasks | Files | Completed |
|------------|----------|-------|-------|-----------|
| 01-01 | 6 min | 2 | 13 | 2026-04-20 |
| 01-02 | 4 min | 3 | 6 | 2026-04-20 |
| 01-03 | 5 min | 3 | 4 | 2026-04-20 |
| 01-04 | 5 min | 2 + human checkpoint (approved) | 6 | 2026-04-21 |
| 02-01 | 4 min | 2 | 45 (42 new stubs + 3 modified) | 2026-04-21 |

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
| 14 Phase 2 utility crates scaffolded up-front with stub uumain | Wave 2/3/4 plans run in parallel; workspace members list must resolve before any of them run. Stubs print `{name}: not yet implemented` and exit 1, except gow-true/gow-false which already implement final 0/1 behavior per D-22 (Plan 02-01) |
| Stubs intentionally omit build.rs | Each utility's dedicated plan (Wave 2/3/4) adds embed-manifest build.rs alongside the real uumain in a single commit — keeps "crate becomes real" as one atomic change (Plan 02-01) |
| .claude/ added to .gitignore | Claude Code agent-local settings + worktree scaffolding are environment-specific tooling, not project artifacts. Ignoring keeps `git status` clean for remaining Phase 2 plans (Plan 02-01) |

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

- Executed Phase 2 Plan 01 (workspace prep + 14 stub utility crates)
  - Task 1 (`6911942`): root Cargo.toml — expanded members 2→16, added snapbox 1.2 / bstr 1 / filetime 0.2 to [workspace.dependencies]
  - Task 2 (`36053fd`): 14 × 3 = 42 stub files (Cargo.toml + src/lib.rs + src/main.rs per crate); gow-true/gow-false already final per D-22; 12 stubs print `{name}: not yet implemented` and exit 1
- Build/test gate: `cargo build --workspace` clean (0 warnings); `cargo clippy --workspace -- -D warnings` clean; `cargo test --workspace` 34 + 9 + 3 doctests pass, no regressions
- 1 auto-fixed deviation (Rule 2 hygiene: added `.claude/` to `.gitignore`)

### What To Do Next

Phase 2 Wave 2 begins. Plans 02-02 (true/false/yes), 02-03 (echo), 02-04 (pwd), 02-05 (basename/dirname) are all unblocked by this plan — they can execute in parallel because each touches only its own crate's `Cargo.toml`, `src/`, and `tests/` directories. Root Cargo.toml is now frozen for the rest of Phase 2 (no more member additions needed). Each wave 2 plan must replace its crate's stub uumain AND add a build.rs copying the gow-probe template verbatim.

---

*State initialized: 2026-04-20*
*Plan 01-01 completed: 2026-04-20*
*Plans 01-02, 01-03 completed: 2026-04-20*
*Plan 01-04 + Phase 1 verification completed: 2026-04-21*
*Plan 02-01 completed: 2026-04-21*
