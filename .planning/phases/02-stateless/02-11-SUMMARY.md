---
phase: 02-stateless
plan: 11
subsystem: utility-windows-native
tags: [which, path, pathext, env, resolver, windows, gow-276, clap, assert_cmd, tempfile]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: "gow_core::init (UTF-8 console, VT mode), gow_core::args::parse_gnu (GNU-style exit-1 wrapper)"
  - phase: 02-stateless / 02-01
    provides: "gow-which stub crate registered in workspace, bin/lib split (uu_which)"
provides:
  - "gow-which hybrid PATHEXT resolver (WHICH-01, GOW #276 fixed)"
  - "`which` binary with --all / -a flag"
  - "load_pathext() / parse_pathext_string() reusable helpers (future find / where utilities can consume)"
  - "GOW_PATHEXT env-var test override pattern (D-18d) reusable across any future PATH-aware utility"
affects:
  - "Phase 3 (filesystem utilities) — reuse hybrid-resolver pattern if `find -type x` or `type` are added"
  - "Phase 6 (MSI installer) — `which.exe` is one of the shipped binaries; PATH registration proves end-to-end"

# Tech tracking
tech-stack:
  added: ["embed-manifest (build-dep, Windows UTF-8 manifest)", "assert_cmd", "predicates", "tempfile (dev-deps only — no new runtime dep, resolver is stdlib-only per Q6)"]
  patterns:
    - "Hybrid PATHEXT strategy: literal-first then PATHEXT expansion, per PATH directory (D-18)"
    - "GOW_<UTILITY>_<KNOB> env override pattern for deterministic integration testing (D-18d) — avoids racy #[test] env manipulation"
    - "Pure-parser + env-loader split (`parse_pathext_string` is unit-testable, `load_pathext` is integration-tested via subprocess isolation)"

key-files:
  created:
    - "crates/gow-which/src/pathext.rs — PATHEXT resolver (GOW_PATHEXT > PATHEXT > .COM;.EXE;.BAT;.CMD fallback)"
    - "crates/gow-which/build.rs — Windows manifest embed (UTF-8 active code page, long-path aware)"
    - "crates/gow-which/tests/integration.rs — 13 subprocess tests using GOW_PATHEXT isolation"
  modified:
    - "crates/gow-which/src/lib.rs — uumain + hybrid find() loop"
    - "crates/gow-which/Cargo.toml — add build-dep (embed-manifest) and dev-deps (assert_cmd, predicates, tempfile)"
    - "Cargo.lock — regenerated"

key-decisions:
  - "Hand-rolled resolver instead of the `which` crate (D-18 demands GOW_PATHEXT override for test determinism; black-box `which` crate does not expose PATHEXT plumbing)."
  - "Literal match wins over PATHEXT expansion when both exist in the same PATH directory (D-18 — preserves GNU script compatibility; mirrors how cmd.exe itself resolves)."
  - "Do NOT canonicalize the found path (D-18e — symlink resolution forbidden). Output echoes the PATHEXT casing we constructed (e.g. `foo.EXE`), not the on-disk casing (`foo.exe`). Tests fold case accordingly."
  - "Default PATHEXT fallback uses the classic .COM;.EXE;.BAT;.CMD (D-18a) — matches Windows 2000-era baseline, not modern .CPL/.MSC extended set."
  - "-a returns BOTH the literal match AND every PATHEXT expansion across every PATH dir — maximum discoverability; covered by `test_a_includes_literal_and_pathext` and `test_a_returns_all_matches_across_dirs`."

patterns-established:
  - "Env-var override for test determinism: set `GOW_PATHEXT` (and `.env(\"PATH\", tempdir)`) per assert_cmd invocation; avoids cross-test env races."
  - "Pure parser vs. env loader: keep parsing in a plain function (`parse_pathext_string`) so #[cfg(test)] unit tests can run without touching process env; expose a thin loader (`load_pathext`) that reads env and delegates."
  - "Every gow binary crate has its own build.rs that embeds the Windows manifest (RESEARCH.md Pitfall 4 — must be per-binary, not in gow-core)."

requirements-completed: [WHICH-01]

# Metrics
duration: 6min
completed: 2026-04-21
---

# Phase 02 Plan 11: gow-which Summary

**GNU `which` + Windows PATHEXT hybrid resolver: literal-first per PATH directory then `.COM;.EXE;.BAT;.CMD` expansion, with `GOW_PATHEXT` override for deterministic tests. Closes GOW #276.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-04-21T01:10:41Z
- **Completed:** 2026-04-21T01:16:04Z
- **Tasks:** 2
- **Files modified:** 6 (4 created, 2 modified + Cargo.lock)

## Accomplishments
- Hybrid PATHEXT resolver — literal-match-first, then expansion per PATH dir (D-18).
- `-a` / `--all` flag returns every hit across every PATH directory.
- GOW_PATHEXT > PATHEXT > `.COM;.EXE;.BAT;.CMD` fallback chain (D-18a, D-18d).
- 19 tests pass (6 unit parser tests + 13 integration subprocess tests).
- Zero new runtime dependencies beyond stdlib + clap (Q6 guidance followed — no `which` crate).
- `which cargo` → `C:\Users\...\cargo.EXE`; `which -a cargo` → both cargo.EXE; `which zzz…` → exit 1 with GNU-style `which: no zzz… in (PATH)` message.

## Task Commits

Each task was committed atomically (single-repo, `--no-verify` per parallel-executor protocol):

1. **Task 1: pathext module** — `56f7693` (feat) — `parse_pathext_string` + `load_pathext` + 6 unit tests; `mod pathext;` wired into lib.rs with temporary `#[allow(dead_code)]` guards.
2. **Task 2: hybrid find loop + build.rs + Cargo.toml + integration tests** — `cee6a6b` (feat) — full uumain/find implementation, Windows manifest embed, 13 integration tests, dead-code guards removed.

_No separate TDD RED commit: Task 1's unit tests and Task 2's feature land together because Task 2 is a straight implementation task (tdd="false" — only Task 1 was marked tdd="true" and it bundles RED+GREEN in the same commit since the parser helpers and their tests were written together)._

**Plan metadata commit:** to be added after SUMMARY.md is staged.

## Files Created/Modified
- `crates/gow-which/src/pathext.rs` **(created)** — 70 lines. `parse_pathext_string(&str) -> Vec<OsString>` and `load_pathext()` with `GOW_PATHEXT > PATHEXT > default` precedence; inline 6-test suite for the pure parser.
- `crates/gow-which/src/lib.rs` **(modified)** — 110 lines. `uumain`, `find`, `is_executable_file`, `uu_app` (clap). Hybrid loop iterates `std::env::split_paths` and for each dir tries literal then every PATHEXT extension; `-a` collects all, default short-circuits on first.
- `crates/gow-which/build.rs` **(created)** — 27 lines. embed-manifest for `ActiveCodePage::Utf8` + `long_path_aware`.
- `crates/gow-which/Cargo.toml` **(modified)** — add `[build-dependencies] embed-manifest = "1.5"` and the standard `[dev-dependencies]` trio (assert_cmd, predicates, tempfile).
- `crates/gow-which/tests/integration.rs` **(created)** — 219 lines. 13 assert_cmd subprocess tests, all using `.env("PATH", tempdir)` + `.env("GOW_PATHEXT", …)` for isolation. Covers literal-wins, expansion fallback, -a all, default fallback, CWD not searched, multi-name, partial miss, UTF-8 (Korean) name, no args, bad flag.
- `Cargo.lock` **(modified)** — embed-manifest and rustversion pulled into lock.

## Decisions Made

- **Hand-rolled resolver (not `which` crate).** The plan's runtime notes explicitly forbid the crate because D-18d's `GOW_PATHEXT` override needs direct env control. The `which` crate treats PATHEXT as an internal implementation detail; exposing a test hook would require either forking or monkey-patching the system env (racy). Hand-rolling is ~50 lines of stdlib code — a clear win.
- **Output preserves PATHEXT casing, not disk casing.** Per D-18e we do NOT canonicalize. On case-insensitive NTFS, a candidate `foo.EXE` (from `GOW_PATHEXT=.EXE`) resolves to the real file `foo.exe`, but the printed path reads `foo.EXE`. Tests fold case via a `stdout_ci_contains` helper rather than tightening the resolver.
- **`-a` includes both literal and PATHEXT expansions inside the same directory.** GNU `which -a` returns every match; we extend "match" to cover Windows PATHEXT. `test_a_includes_literal_and_pathext` pins this behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Temporary `#[allow(dead_code)]` on pathext helpers in Task 1 commit**
- **Found during:** Task 1 clippy gate (`cargo clippy -p gow-which --lib -- -D warnings`).
- **Issue:** The plan commits Task 1 before Task 2, so `parse_pathext_string`, `load_pathext`, and `DEFAULT_PATHEXT` briefly have no consumer. Clippy with `-D warnings` rejects dead code.
- **Fix:** Added three `#[allow(dead_code)]` attributes with a top-of-file comment noting they lift once Task 2 wires the consumer. Task 2 removed them; lib.rs now references `pathext::load_pathext()`.
- **Files modified:** `crates/gow-which/src/pathext.rs` (Task 1 add, Task 2 remove).
- **Verification:** Task 1 clippy green; Task 2 clippy green without the allow-attrs.
- **Committed in:** `56f7693` (add) and `cee6a6b` (remove).

**2. [Rule 1 - Bug] Case-insensitive output assertions**
- **Found during:** Task 2 first `cargo test -p gow-which` run — 5 tests failed because the resolver emitted `foo.EXE` (upper) while assertions expected `foo.exe` (lower).
- **Issue:** On NTFS both are the same file, but `predicate::str::contains("foo.exe")` is case-sensitive. Canonicalizing the found path would fix the casing but also resolve symlinks, violating D-18e.
- **Fix:** Added `stdout_ci_contains(needle)` helper in `tests/integration.rs` that folds case before substring search; swapped the 5 affected assertions.
- **Files modified:** `crates/gow-which/tests/integration.rs`.
- **Verification:** All 13 integration tests pass.
- **Committed in:** `cee6a6b` (included in Task 2 commit).

**3. [Rule 3 - Blocking] clippy `just_underscores_and_digits` on `_1`/`_2` bindings**
- **Found during:** Task 2 clippy gate.
- **Issue:** Plan's integration test code used `let _1 = make_executable_file(…); let _2 = …;` to bind tempfile handles. clippy 1.95 flags `_1`/`_2` as non-descriptive.
- **Fix:** Renamed to meaningful anonymous prefixes: `_alpha`/`_beta`, `_a`/`_b`, `_literal`/`_expanded`.
- **Files modified:** `crates/gow-which/tests/integration.rs` (3 test sites).
- **Verification:** `cargo clippy -p gow-which --all-targets -- -D warnings` exits 0.
- **Committed in:** `cee6a6b` (included in Task 2 commit).

---

**Total deviations:** 3 auto-fixed (2 blocking / tooling friction, 1 bug — test-vs-code mismatch).
**Impact on plan:** None architectural. All three are surface adjustments (attribute, predicate helper, identifier rename) that keep the plan's semantics intact. The hybrid-loop algorithm and PATHEXT precedence are verbatim from the plan.

## Issues Encountered

- **PATHEXT casing surprise.** The resolver constructs `dir.join(name) + ext` and prints that path as-is, so the user-supplied PATHEXT casing flows through to stdout. On a case-insensitive filesystem this is harmless for execution but cosmetically surprising (e.g. the user sees `C:\...\foo.EXE` when their PATHEXT is `.EXE` but the binary is `foo.exe`). Documented in `Decisions Made` above. Users who want real-case output can pipe through `cmd /c dir` or can override PATHEXT to match disk casing. An opt-in `--canonicalize` flag could be revisited in v2 but was explicitly deferred per D-18e.

## GOW #276 Regression Coverage

The long-standing GOW #276 bug — `which foo` couldn't find `foo.exe` — is now pinned by **`test_pathext_expansion_fallback`**: it creates only `foo.exe` in a tempdir, sets `GOW_PATHEXT=.EXE;.BAT`, runs `which foo`, and asserts the stdout contains `foo.exe` with exit 0. Without the PATHEXT expansion phase of the hybrid loop, this test fails deterministically. Combined with `test_literal_beats_pathext_expansion` (both `foo` and `foo.exe` present → literal wins) and `test_a_includes_literal_and_pathext` (`-a` surfaces both), the regression is triangulated.

## Sample output

```
> which cargo
C:\Users\노명훈\.cargo\bin\cargo.EXE

> which -a cargo
C:\Users\노명훈\.cargo\bin\cargo.EXE
C:\Users\노명훈\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.EXE

> which zzzz-nonexistent-xyz ; echo EXIT=$?
which: no zzzz-nonexistent-xyz in (…PATH…)
EXIT=1
```

## User Setup Required

None — `which` is pure-stdlib plus clap. No external services, no env vars required by end users (`GOW_PATHEXT` is test-only and undocumented for end users).

## Next Phase Readiness

- `gow-which` ROADMAP criterion 3 delivered and testable.
- Pattern reusable: any future utility needing PATH traversal (e.g. `whereis`, `type`) can reuse `pathext::load_pathext`. Consider promoting the module to `gow-core::pathext` if a second consumer appears.
- No blockers for Wave-4 siblings.

## TDD Gate Compliance

Plan is `type: execute`, not `type: tdd` (plan-level). Task 1 is `tdd="true"` — the parser helpers landed with their 6 unit tests in the same commit (RED and GREEN fused since the parser was trivial; this is acceptable for pure-function parsers where the "failing first" gate adds no real design pressure). The plan's success-criteria checklist does not require separate RED/GREEN commits.

## Self-Check: PASSED

Files referenced above verified present:

- `crates/gow-which/src/pathext.rs` — FOUND
- `crates/gow-which/src/lib.rs` — FOUND (modified)
- `crates/gow-which/build.rs` — FOUND
- `crates/gow-which/Cargo.toml` — FOUND (modified)
- `crates/gow-which/tests/integration.rs` — FOUND

Commits referenced verified in `git log --oneline`:

- `56f7693` — feat(02-11): add pathext module for gow-which PATHEXT resolution — FOUND
- `cee6a6b` — feat(02-11): implement gow-which hybrid PATHEXT resolver (WHICH-01, GOW #276) — FOUND

Gate commands (re-verified post-commit):
- `cargo build -p gow-which` → success
- `cargo test -p gow-which` → 6 unit + 13 integration + 0 doc = 19 passed, 0 failed
- `cargo clippy -p gow-which --all-targets -- -D warnings` → clean

---
*Phase: 02-stateless*
*Plan: 11*
*Completed: 2026-04-21*
