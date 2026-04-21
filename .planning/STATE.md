---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: phase_complete
last_updated: "2026-04-21T01:45:00Z"
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 15
  completed_plans: 15
  percent: 33
---

# Project State: gow-rust

**Last updated:** 2026-04-21
**Session:** Phase 2 (Stateless Utilities) execution complete and verified — all 14 utilities pass GNU compatibility gates on Windows

---

## Project Reference

**Core Value:** Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

**Current Focus:** Phases 1 and 2 verified complete. Next is Phase 3 (Filesystem Utilities — ls, cat, cp, mv, rm, ln, chmod, head, tail -f, dos2unix, unix2dos).

---

## Current Position

**Phase:** 2 (Stateless Utilities) — COMPLETE
**Plan:** All 11 Phase 2 plans complete; verifier passed (19/19 must-haves, 14/14 requirements, 5/5 ROADMAP success criteria). Next: Phase 3 (Filesystem Utilities) — start with `/gsd-discuss-phase 3`.
**Status:** Phase complete

**Progress:**

```
Phase 1 [##########] 100% Foundation ✓ (4/4 plans, 10/10 requirements verified)
Phase 2 [##########] 100% Stateless Utilities ✓ (11/11 plans, 14/14 requirements verified)
Phase 3 [          ] 0%   Filesystem Utilities
Phase 4 [          ] 0%   Text Processing
Phase 5 [          ] 0%   Search and Navigation
Phase 6 [          ] 0%   Archive, Compression, and Network
```

**Overall:** 2/6 phases complete (24/60 requirements verified, 15/15 plans complete so far)

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases total | 6 |
| Phases complete | 2 |
| Requirements total | 59 |
| Requirements complete | 24 (Phase 1: FOUND-01..07, WIN-01..03; Phase 2: UTIL-01..09, TEXT-03, FILE-06..08, WHICH-01) |
| Plans created | 15 (4 Phase 1 + 11 Phase 2) |
| Plans complete | 15 (all Phase 1 + all Phase 2) |
| Tests passing | 265 (34 gow-core unit + 9 gow-probe integration + 3 doctests + 219 Phase 2 utility tests) |
| Utilities delivered | 14 (echo, pwd, env, tee, basename, dirname, yes, true, false, mkdir, rmdir, touch, wc, which) |

### Per-plan execution

| Phase-Plan | Duration | Tasks | Files | Completed |
|------------|----------|-------|-------|-----------|
| 01-01 | 6 min | 2 | 13 | 2026-04-20 |
| 01-02 | 4 min | 3 | 6 | 2026-04-20 |
| 01-03 | 5 min | 3 | 4 | 2026-04-20 |
| 01-04 | 5 min | 2 + human checkpoint (approved) | 6 | 2026-04-21 |
| 02-01 | 4 min | 2 | 45 (42 new stubs + 3 modified) | 2026-04-21 |
| 02-02 | 7 min | 2 (TDD split) | 10 | 2026-04-21 |
| 02-03 | 6 min | 2 | 5 | 2026-04-21 |
| 02-04 | 3 min | 2 | 5 | 2026-04-21 |
| 02-05 | 3.5 min | 2 | 8 | 2026-04-21 |
| 02-06 | 6 min | 2 | 8 | 2026-04-21 |
| 02-07 | 15 min | 2 | 5 | 2026-04-21 |
| 02-08 | 6 min | 2 (TDD split) | 4 | 2026-04-21 |
| 02-09 | 35 min | 2 (TDD split) | 5 | 2026-04-21 |
| 02-10 | 9 min | 2 (TDD split) | 6 | 2026-04-21 |
| 02-11 | 6 min | 2 | 5 | 2026-04-21 |

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

- Executed Phase 2 across 4 waves with wave-based parallel worktree execution:
  - Wave 1 (02-01): workspace prep + 14 stub crates
  - Wave 2 (02-02..05): gow-true/false/yes, gow-echo, gow-pwd, gow-basename/dirname — 4 parallel worktrees
  - Wave 3 (02-06..08): gow-mkdir/rmdir, gow-tee, gow-wc — 3 parallel worktrees
  - Wave 4 (02-09..11): gow-env, gow-touch, gow-which — 3 parallel worktrees (most complex utilities)
- 14 utility binaries delivered: echo.exe, pwd.exe, env.exe, tee.exe, basename.exe, dirname.exe, yes.exe, true.exe, false.exe, mkdir.exe, rmdir.exe, touch.exe, wc.exe, which.exe (GNU names per D-14)
- GOW #133 (mkdir -p) and GOW #276 (which PATHEXT) explicitly resolved
- Pre-existing gow-core test clippy warnings fixed inline during phase verification
- gsd-verifier returned status=passed: 19/19 must-haves, 14/14 requirements, 5/5 ROADMAP success criteria
- `cargo test --workspace` 265 passed; `cargo clippy --workspace --all-targets -- -D warnings` clean

### What To Do Next

Phase 2 verified complete. Recommended next: `/gsd-discuss-phase 3` to gather context on Phase 3 (Filesystem Utilities) — ls, cat, cp, mv, rm, ln, chmod, head, tail -f, dos2unix, unix2dos. Phase 3 is larger in scope than Phase 2 because it includes stateful operations (tail -f via notify/ReadDirectoryChangesW, sed -i-style atomic writes, symlink/junction handling via gow_core::fs::LinkType already built in Phase 1).

---

*State initialized: 2026-04-20*
*Plan 01-01 completed: 2026-04-20*
*Plans 01-02, 01-03 completed: 2026-04-20*
*Plan 01-04 + Phase 1 verification completed: 2026-04-21*
*Plan 02-01 completed: 2026-04-21*
*Plans 02-02..02-11 + Phase 2 verification completed: 2026-04-21*
