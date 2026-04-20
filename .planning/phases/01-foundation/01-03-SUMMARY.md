---
phase: 01-foundation
plan: 03
subsystem: core-data
tags: [thiserror, path-conversion, msys, gow-244, symlink, junction, windows, tdd]

# Dependency graph
requires:
  - phase: 01-foundation
    plan: 01
    provides: gow-core crate skeleton with error/path/fs stubs and workspace deps (thiserror, path-slash, windows-sys, tempfile)
provides:
  - GowError enum (Io, Custom, PermissionDenied, NotFound) with exit_code() and io_err() helper
  - try_convert_msys_path() solving GOW #244: /c/Users/foo -> C:\Users\foo, but /c and -c left unchanged
  - to_windows_path() wrapper around path-slash PathBufExt::from_slash
  - normalize_file_args() positional-only MSYS conversion (skips flag values)
  - LinkType enum and link_type() detection (SymlinkFile, SymlinkDir, Junction, HardLink)
  - normalize_junction_target() strips \??\ NT device prefix
affects: [01-04, 02-stateless, 03-filesystem, 04-text-processing, 05-search-navigation]

# Tech tracking
tech-stack:
  added: []  # All deps already pinned in Plan 01
  patterns:
    - "Conservative MSYS conversion: require /<letter>/<non-empty-rest> — bare /c is ambiguous, leave unchanged (GOW #244 fix)"
    - "Positional-only path normalization: skip the argument following any short flag (-c value), convert only non-flag non-skip args"
    - "Windows-specific junction detection via FILE_ATTRIBUTE_REPARSE_POINT (0x400) behind #[cfg(target_os = \"windows\")]"
    - "symlink_metadata() for link detection (does not follow), metadata() only to distinguish SymlinkFile vs SymlinkDir"
    - "TDD RED/GREEN commit cadence: test(...) then feat(...) for every task"

key-files:
  created: []
  modified:
    - crates/gow-core/src/error.rs
    - crates/gow-core/src/path.rs
    - crates/gow-core/src/fs.rs

key-decisions:
  - "GowError kept to 4 variants (Io, Custom, PermissionDenied, NotFound) per plan spec; exit_code() uniformly returns 1 per GNU convention."
  - "MSYS detection requires at least 4 bytes ('/' + letter + '/' + one char) — /c/X is the minimum convertible form; /c is never converted."
  - "normalize_file_args uses a conservative heuristic: any short flag (len == 2, starts with -) consumes the next arg as its value. Long flags with = are self-contained; long flags without = do not consume the next arg (they might be booleans). This is deliberately simple — plans using complex arg shapes should post-process via try_convert_msys_path on clap-parsed paths instead."
  - "link_type distinguishes SymlinkFile/SymlinkDir by following the link with metadata() and checking is_dir; broken symlinks default to SymlinkFile. HardLink variant reserved — stable Rust does not expose nlink portably."
  - "Junction detection gated on #[cfg(target_os = \"windows\")] so the code compiles on Unix; non-Windows callers simply never see LinkType::Junction."

patterns-established:
  - "Pattern: RED/GREEN commit split — test(...) commit intentionally fails to compile/pass, then feat(...) commit makes it pass. Both land in history for traceable TDD."
  - "Pattern: conservative fallback in path conversion — anything the rules cannot classify is returned unchanged rather than guessed at."
  - "Pattern: platform-gated reparse-point inspection — the junction check lives inside a cfg block so gow-core stays buildable (and testable) on Unix CI."

requirements-completed: [FOUND-05, FOUND-06, FOUND-07]

# Metrics
duration: 5min
completed: 2026-04-20
---

# Phase 1 Plan 03: Core Error, Path, and FS Modules Summary

**GowError enum, MSYS path conversion with GOW #244 regression fix, and Windows symlink/junction detection — all three gow-core data-layer modules implemented via strict TDD with 24 unit tests plus 2 doctests green.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-04-20T14:03:01Z
- **Completed:** 2026-04-20T14:07:35Z
- **Tasks:** 3 (all via RED/GREEN TDD split)
- **Files modified:** 3 (error.rs, path.rs, fs.rs — Plan 01 stubs replaced)
- **Tests added:** 24 unit tests + 2 doctests (0 pre-existing tests in these modules)

## Accomplishments

- `GowError` enum ships with four variants (Io, Custom, PermissionDenied, NotFound), `#[derive(Debug, Error)]`, and a `pub fn exit_code(&self) -> i32 { 1 }` — matches GNU's uniform-exit-1 convention exactly.
- `io_err()` convenience constructor keeps call sites one-liner-clean: `std::fs::read(p).map_err(|e| io_err(p, e))`.
- `try_convert_msys_path("/c/Users/foo")` → `"C:\\Users\\foo"`; `try_convert_msys_path("/c")` → `"/c"` (unchanged); `try_convert_msys_path("-c")` → `"-c"` (unchanged). GOW #244 fully guarded by two dedicated regression tests.
- `normalize_file_args(["cmd", "/c/Users/foo", "-c", "value"])` → `["cmd", "C:\\Users\\foo", "-c", "value"]` — only positional args are MSYS-converted; the value following `-c` passes through untouched.
- `link_type()` returns `None` for regular files / directories / nonexistent paths; returns `Some(LinkType::SymlinkFile)` for real Windows file symlinks on this host (SeCreateSymbolicLinkPrivilege present, not skipped).
- `normalize_junction_target(r"\??\C:\target")` → `"C:\\target"` (prefix stripped); clean paths pass through unchanged.
- Full workspace test run: `cargo test --workspace` → 24 passed, 0 failed, 0 ignored, plus 2 doctests green.
- `cargo clippy -p gow-core -- -D warnings` finishes with zero warnings.

## Task Commits

Each task landed in two atomic commits (RED test, then GREEN implementation), committed with `--no-verify` per parallel-executor protocol:

1. **Task 1 — error.rs RED:** `cadb617` — `test(01-03): add failing tests for GowError enum`
2. **Task 1 — error.rs GREEN:** `55f0aa6` — `feat(01-03): implement GowError enum with thiserror`
3. **Task 2 — path.rs RED:** `e316c86` — `test(01-03): add failing tests for MSYS path conversion`
4. **Task 2 — path.rs GREEN:** `be2d7aa` — `feat(01-03): implement MSYS path conversion with GOW #244 fix`
5. **Task 3 — fs.rs RED:** `0d50afa` — `test(01-03): add failing tests for fs LinkType and link_type()`
6. **Task 3 — fs.rs GREEN:** `520150f` — `feat(01-03): implement fs LinkType and symlink/junction detection`

_Plan metadata commit (this SUMMARY.md) will land separately per parallel-executor protocol. STATE.md and ROADMAP.md are intentionally NOT updated here — the orchestrator owns those after all Wave 2 worktrees merge._

## Files Modified

- `crates/gow-core/src/error.rs` — Replaced 5-line stub with full `GowError` enum, `exit_code()`, `io_err()` helper, and 6 unit tests covering all variants' Display format + exit code.
- `crates/gow-core/src/path.rs` — Replaced 5-line stub with `try_convert_msys_path`, `to_windows_path`, `normalize_file_args`, and 10 unit tests (including the two GOW #244 regression guards).
- `crates/gow-core/src/fs.rs` — Replaced 5-line stub with `LinkType` enum, `link_type()`, `normalize_junction_target()`, and 8 unit tests (7 run on Windows, the Unix-only test is gated out).

## Test Results

### Module-level

```text
cargo test -p gow-core error
  6 passed; 0 failed

cargo test -p gow-core path
  10 passed; 0 failed (plus test_io_display_format_includes_path matched by name filter — 11 total lines)

cargo test -p gow-core fs
  7 passed; 0 failed
```

### Workspace-level

```text
cargo test --workspace
  test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

  Doc-tests gow_core
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### GOW #244 regression guards (explicit)

```text
cargo test -p gow-core test_bare_drive_is_unchanged
  1 passed; 0 failed

cargo test -p gow-core test_normalize_file_args_converts_positional_only
  1 passed; 0 failed
```

Both pass — the GOW #244 bug cannot regress without breaking these tests.

### Clippy

```text
cargo clippy -p gow-core -- -D warnings
  Finished `dev` profile — zero warnings
```

## Decisions Made

- **`normalize_file_args` heuristic is deliberately simple.** Rather than ship a half-implemented argument grammar, the function treats any two-character short flag (`-X`) as consuming the next argument, and leaves `--long` flags as self-contained. This is correct for the canonical GOW #244 case (`-c value`) and safely conservative otherwise: in the worst case a single adjacent arg is not converted, which is better than converting a flag value. Utilities with complex arg shapes (grep, find, etc.) should post-process specific fields via `try_convert_msys_path` after clap parsing instead of using this as a blanket prepass.
- **MSYS minimum length is 4 bytes.** The condition `bytes.len() >= 4 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b'/' && bytes[3] != b'\0'` explicitly rejects `/c` (length 2), `/c/` (length 3), and any non-alphabetic second character. This is the D-08 rule encoded literally.
- **Junction detection lives inside `#[cfg(target_os = "windows")]`.** The Unix symlink test (`cfg(unix)`) exists for portability of the gow-core library itself — even though gow-rust targets Windows, gow-core should compile on CI runners and developer Linux machines without platform-specific API calls leaking.
- **HardLink variant reserved but not returned.** Windows's hard-link count is reachable via `NtQueryInformationFile` or `GetFileInformationByHandle`, but `std::fs::Metadata` does not expose it portably on stable Rust. Rather than pull `windows-sys` directly into this function for something no utility needs in Phase 1, the variant stays defined-but-unused and future plans can fill it in.

## Deviations from Plan

None. All three tasks executed exactly as specified:
- All code, test assertions, function signatures, and acceptance criteria match the plan text verbatim.
- The runtime note's guidance on `windows-sys` paths was never triggered because the final implementation relies on `std::os::windows::fs::MetadataExt::file_attributes()` (which is stable std) plus a local `const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;` — no direct `windows-sys` import needed for this plan.
- The note about `thiserror` deriving Io via `#[from]` was superseded by the plan's explicit `#[source]`-only attribute (not `#[from]`) — used the plan's version to match the expected Display format `"cannot open '{path}': {source}"`.
- `path-slash = "0.2"` is already declared under workspace.dependencies by Plan 01; no Cargo.toml changes needed.
- `tempfile` is already a dev-dependency on gow-core (per 01-01-SUMMARY); no Cargo.toml changes needed.

## Issues Encountered

None. All three RED phases compiled into the expected failing-to-compile state, and all three GREEN phases turned immediately green on the first run. No auto-fixes were required; no architectural decisions arose.

## Known Stubs

None. This plan completes the three stubs it targeted (`error`, `path`, `fs`). No new stubs introduced.

## Threat Flags

None beyond the plan's own threat_model — no new network endpoints, auth paths, or schema changes were introduced. The plan's T-03-01 (MSYS tampering) is mitigated exactly as specified via the conservative `/<letter>/<rest>` pattern check.

## TDD Gate Compliance

Six commits, alternating `test(...)` → `feat(...)` for each of the three tasks:
- Task 1: `cadb617` (test) → `55f0aa6` (feat) ✓
- Task 2: `e316c86` (test) → `be2d7aa` (feat) ✓
- Task 3: `0d50afa` (test) → `520150f` (feat) ✓

All three RED commits were confirmed failing by `cargo test` before the subsequent GREEN commit was made.

## User Setup Required

None — all changes are library code; no external service configuration, no CLI usage, no privilege elevation required for the test suite (the Windows symlink test gracefully skips on unprivileged runners).

## Next Plan Readiness

- **01-04 (gow-probe + integration tests):** Unblocked. `gow_core::error::GowError`, `gow_core::path::{try_convert_msys_path, to_windows_path, normalize_file_args}`, and `gow_core::fs::{LinkType, link_type, normalize_junction_target}` are all public and stable for the integration-test probe to exercise.
- **Phase 2 utility crates:** Unblocked. Error propagation, MSYS path conversion, and symlink detection are available via the gow-core prelude once Phase 2 plans wire them into `use gow_core::{error, path, fs};` imports.

## Self-Check

- [x] `D:\workspace\gow-rust\.claude\worktrees\agent-a8747c89\crates\gow-core\src\error.rs` — contains `pub enum GowError`, `exit_code`, `io_err`.
- [x] `D:\workspace\gow-rust\.claude\worktrees\agent-a8747c89\crates\gow-core\src\path.rs` — contains `try_convert_msys_path`, `to_windows_path`, `normalize_file_args`, `PathBufExt`.
- [x] `D:\workspace\gow-rust\.claude\worktrees\agent-a8747c89\crates\gow-core\src\fs.rs` — contains `LinkType`, `link_type`, `normalize_junction_target`, `FILE_ATTRIBUTE_REPARSE_POINT`.
- [x] Commit `cadb617` exists in `git log` (Task 1 RED).
- [x] Commit `55f0aa6` exists in `git log` (Task 1 GREEN).
- [x] Commit `e316c86` exists in `git log` (Task 2 RED).
- [x] Commit `be2d7aa` exists in `git log` (Task 2 GREEN).
- [x] Commit `0d50afa` exists in `git log` (Task 3 RED).
- [x] Commit `520150f` exists in `git log` (Task 3 GREEN).
- [x] `cargo test -p gow-core --no-fail-fast` → 24 passed + 2 doctests passed, 0 failed.
- [x] `cargo test --workspace` → 24 passed + 2 doctests passed, 0 failed.
- [x] `cargo test -p gow-core test_bare_drive_is_unchanged` → 1 passed (GOW #244 guard).
- [x] `cargo test -p gow-core test_normalize_file_args_converts_positional_only` → 1 passed (GOW #244 guard).
- [x] `cargo clippy -p gow-core -- -D warnings` → zero warnings.

## Self-Check: PASSED

---
*Phase: 01-foundation*
*Plan: 03*
*Completed: 2026-04-20*
