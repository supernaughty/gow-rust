---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-04-20T13:32:48.123Z"
progress:
  total_phases: 6
  completed_phases: 0
  total_plans: 4
  completed_plans: 0
  percent: 0
---

# Project State: gow-rust

**Last updated:** 2026-04-20
**Session:** Initial roadmap creation

---

## Project Reference

**Core Value:** Windows 사용자가 별도의 무거운 환경(WSL, Cygwin) 없이 GNU 명령어를 네이티브 성능으로 사용할 수 있어야 한다.

**Current Focus:** Phase 1 — Foundation (gow-core shared library and Cargo workspace)

---

## Current Position

**Phase:** 1 (Foundation)
**Plan:** None started
**Status:** Ready to execute

**Progress:**

```
Phase 1 [          ] 0%   Foundation
Phase 2 [          ] 0%   Stateless Utilities
Phase 3 [          ] 0%   Filesystem Utilities
Phase 4 [          ] 0%   Text Processing
Phase 5 [          ] 0%   Search and Navigation
Phase 6 [          ] 0%   Archive, Compression, and Network
```

**Overall:** 0/6 phases complete

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases total | 6 |
| Phases complete | 0 |
| Requirements total | 59 |
| Requirements complete | 0 |
| Plans created | 0 |
| Plans complete | 0 |

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

- Read PROJECT.md, REQUIREMENTS.md, research/SUMMARY.md, config.json
- Extracted all 59 v1 requirements across 17 categories
- Derived 6 phases from natural dependency boundaries
- Validated 100% requirement coverage (59/59 mapped)
- Wrote ROADMAP.md with phase details and success criteria
- Wrote STATE.md (this file)
- Updated REQUIREMENTS.md traceability section

### What To Do Next

Run `/gsd-plan-phase 1` to break Phase 1 (Foundation) into executable plans.

---

*State initialized: 2026-04-20*
