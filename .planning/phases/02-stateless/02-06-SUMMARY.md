---
phase: 02-stateless
plan: 06
subsystem: stateless-utilities
tags: [mkdir, rmdir, file-utilities, msys-path, file-06, file-07, wave-3, gow-133]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: gow_core::path::try_convert_msys_path (MSYS /c/Users → C:\Users conversion)
  - phase: 01-foundation
    provides: gow_core::args::parse_gnu (clap wrapper with exit-code-1 on bad flags)
  - phase: 02-stateless
    plan: 01
    provides: gow-mkdir + gow-rmdir stub crates already listed in workspace members
provides:
  - crates/gow-mkdir (binary `mkdir.exe` + lib uu_mkdir) — FILE-06
  - crates/gow-rmdir (binary `rmdir.exe` + lib uu_rmdir) — FILE-07
  - GOW issue #133 resolution (mkdir -p idempotent on existing nested paths)
  - Pattern proof: std::fs + multi-operand + MSYS pre-convert extends from read-only utils (basename/dirname) to mutating filesystem ops
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "std::fs delegation: `create_dir_all` handles POSIX `-p` idempotency natively; no custom loop (D-27, RESEARCH.md Q5)"
    - "Manual parent-walk loop for rmdir -p: leaf first, then iterate Path::parent() and stop on ErrorKind::DirectoryNotEmpty (D-28)"
    - "Defense-in-depth `is_not_empty`: primary check via `ErrorKind::DirectoryNotEmpty` (stable in Rust 1.83+); fallback to raw_os_error 145 (Windows) / 39 (Unix)"
    - "MSYS pre-convert at operand ingestion (reused from Plan 02-05): `let converted = gow_core::path::try_convert_msys_path(op);` first line inside per-operand loop"
    - "Per-crate `[build-dependencies] embed-manifest = \"1.5\"` (mirrors gow-probe / gow-echo / gow-basename / gow-dirname pattern)"

key-files:
  created:
    - crates/gow-mkdir/build.rs
    - crates/gow-mkdir/tests/integration.rs
    - crates/gow-rmdir/build.rs
    - crates/gow-rmdir/tests/integration.rs
    - .planning/phases/02-stateless/02-06-SUMMARY.md
  modified:
    - crates/gow-mkdir/Cargo.toml (added [build-dependencies] embed-manifest, [dev-dependencies] assert_cmd/predicates/tempfile)
    - crates/gow-mkdir/src/lib.rs (stub replaced with full uumain delegating to create_dir_all / create_dir)
    - crates/gow-rmdir/Cargo.toml (added [build-dependencies] embed-manifest, [dev-dependencies] assert_cmd/predicates/tempfile)
    - crates/gow-rmdir/src/lib.rs (stub replaced with full uumain + rmdir_parents + is_not_empty)

decisions:
  - "mkdir -m MODE is DROPPED in v1 — Windows has no POSIX mode bits and GNU allows the flag to be a no-op on such platforms. Passing `-m` today produces the standard 'unrecognized option' error, which is the conservative GNU-compatible failure mode. Deferred to a later plan that can wire NTFS ACLs if a user actually demands it. Rationale: the PLAN.md Task 1 note explicitly sanctioned either dropping or silently-accepting `-m`; dropping is chosen so the behavior is discoverable via `--help` and unexpected MODE args never silently vanish."
  - "mkdir -p delegates to std::fs::create_dir_all (no custom loop) — Rust std already implements POSIX idempotency (docs: 'If the path already points to an existing directory, this function returns Ok'). GOW #133 is therefore fixed by delegation alone; no special-case code in uu_mkdir. RESEARCH.md Q5 evaluates and rejects the uutils stack-overflow-prevention rationale (Rust std is loop-based, not recursive)."
  - "rmdir is_not_empty uses both kind() and raw_os_error() — the kind() check is primary (ErrorKind::DirectoryNotEmpty is stable in Rust 1.85, matches on both Windows and Unix), and the raw_os_error() check (145 / 39) is a defense-in-depth layer in case a future std release remaps the error classification. This matches the RESEARCH.md Q5 guidance verbatim."
  - "Per-crate `[build-dependencies] embed-manifest = \"1.5\"` duplicated in gow-mkdir and gow-rmdir Cargo.toml rather than hoisted to workspace.dependencies — mirrors the established gow-probe / gow-echo / gow-basename / gow-dirname pattern from Waves 1-2. Hoisting would require touching workspace Cargo.toml which is out of scope for this plan."

metrics:
  duration: "~6 minutes"
  completed: "2026-04-21"
  tasks_completed: 2
  files_created: 5
  files_modified: 4
  commits: 2
---

# Phase 02 Plan 06: gow-mkdir + gow-rmdir Summary

**One-liner:** Replaced two stub crates with GNU-compatible `mkdir` and `rmdir` binaries that pre-convert MSYS paths and then delegate to `std::fs::create_dir_all` (mkdir -p) and a manual `remove_dir` parent-walk loop with `ErrorKind::DirectoryNotEmpty` guard (rmdir -p) — resolving GOW issue #133 as a no-op of `create_dir_all`'s POSIX-correct idempotency contract.

## Objective

Deliver FILE-06 (`mkdir`) and FILE-07 (`rmdir`) together because they share the std::fs + multi-operand + MSYS pre-convert pattern established in Plan 02-05 (basename/dirname), and FILE-06 is literally a 3-line `create_dir_all` call — bundling prevents over-fragmentation of Wave 3.

The chief behavioral requirement: `mkdir -p a/b/c && mkdir -p a/b/c` must BOTH exit 0 (ROADMAP criterion 4 / GOW issue #133). Rust's `std::fs::create_dir_all` already implements this POSIX contract, so no custom recovery code is needed.

## What Was Built

### Task 1 — gow-mkdir (commit `e6e1c8a`)

**Files:** `crates/gow-mkdir/{Cargo.toml, build.rs, src/lib.rs, tests/integration.rs}`

- `Cargo.toml` gains `[build-dependencies] embed-manifest = "1.5"` and `[dev-dependencies]` for `assert_cmd` / `predicates` / `tempfile`.
- `build.rs` is the canonical gow-probe template (UTF-8 active codepage + long path aware Windows manifest, WIN-01/02).
- `src/lib.rs` replaces the "not yet implemented" stub with:
  - `pub fn uumain` — clap parsing via `gow_core::args::parse_gnu` (short `-p`/`-v`, long `--parents`/`--verbose`), missing-operand guard, per-operand loop.
  - Each operand is run through `gow_core::path::try_convert_msys_path` before the filesystem call (D-26).
  - `-p` branch calls `std::fs::create_dir_all(path)` — POSIX-correct: returns Ok on existing directory, error on existing regular file, creates missing parents.
  - Default branch calls `std::fs::create_dir(path)` — single-level create, errors on missing parent or existing target.
  - `-v` branch prints `mkdir: created directory '{path}'` to stdout on each success (GNU format).
  - Error branch prints `mkdir: cannot create directory '{path}': {error}` to stderr, sets exit code 1, keeps processing remaining operands (GNU multi-arg semantics).
  - `fn uu_app()` — clap `Command` builder with `operands` as `Append + trailing_var_arg`.
- `tests/integration.rs` — 9 `assert_cmd` tests:
  1. `test_create_single_directory` — basic creation
  2. `test_create_existing_without_p_fails` — exit 1 on target exists
  3. `test_p_creates_nested` — FILE-06 / ROADMAP criterion 4
  4. `test_p_is_idempotent` — double `mkdir -p` on same path (GOW #133 regression guard)
  5. `test_verbose_prints_created_line` — `-v` stdout contract
  6. `test_multiple_operands` — multi-arg loop
  7. `test_no_args_error` — exit 1 with "missing operand"
  8. `test_bad_flag_exits_1` — D-02 mapping (clap 2 → 1)
  9. `test_utf8_directory_name` — Dimension 2 (UTF-8 round-trip)

### Task 2 — gow-rmdir (commit `ea20291`)

**Files:** `crates/gow-rmdir/{Cargo.toml, build.rs, src/lib.rs, tests/integration.rs}`

- Same Cargo.toml + build.rs shape as gow-mkdir.
- `src/lib.rs` replaces stub with:
  - `pub fn uumain` — clap parsing (`-p`/`--parents`, `-v`/`--verbose`), missing-operand guard, per-operand loop.
  - Each operand is MSYS-converted before the filesystem call.
  - Default branch: `remove_one(path, verbose)` — single `std::fs::remove_dir` call.
  - `-p` branch: `rmdir_parents(path, verbose)` — removes the leaf, then iterates `Path::parent()` upward, removing each ancestor, stopping gracefully (returning Ok) at the first non-empty parent.
  - `fn is_not_empty(e: &io::Error)` — detects directory-not-empty errors:
    - Primary: `e.kind() == io::ErrorKind::DirectoryNotEmpty` (stable in Rust 1.83+).
    - Fallback: `e.raw_os_error() == Some(145)` on Windows (ERROR_DIR_NOT_EMPTY), `Some(39)` elsewhere (ENOTEMPTY).
  - Error branch prints `rmdir: failed to remove '{path}': {error}` (GNU format).
  - Two `#[cfg(test)]` unit tests validate the `is_not_empty` classifier independently of the filesystem.
- `tests/integration.rs` — 8 `assert_cmd` tests:
  1. `test_remove_empty_directory` — basic removal
  2. `test_remove_nonempty_fails` — exit 1 with `rmdir:` stderr, dir preserved
  3. `test_p_removes_parent_chain` — full empty chain removal
  4. `test_p_stops_at_nonempty_parent` — **critical D-28 regression guard** (parent with sibling file NOT removed)
  5. `test_verbose_prints_removed_line` — `-v` stdout contract
  6. `test_no_args_error` — exit 1 with "missing operand"
  7. `test_bad_flag_exits_1` — D-02 mapping
  8. `test_utf8_directory_name` — Dimension 2

## Verification Evidence

### Intended verification commands (from PLAN.md)

```
cargo build -p gow-mkdir -p gow-rmdir
cargo test  -p gow-mkdir -p gow-rmdir
cargo clippy -p gow-mkdir -p gow-rmdir -- -D warnings

# Manual smoke:
cargo run -p gow-mkdir -- -p test_phase2_p/a/b/c
cargo run -p gow-mkdir -- -p test_phase2_p/a/b/c   # idempotent — second run also exits 0
cargo run -p gow-rmdir -- -p test_phase2_p/a/b/c   # removes c, b, a
```

### Execution-time automation gap (Rule 3 handling)

**The execution environment for this parallel executor agent denied Bash invocations of `cargo` (build, test, check, clippy) — every attempt returned `Permission to use Bash has been denied`.** Other Bash commands (`git`, `ls`) executed normally; only cargo was blocked. Because the plan's verification step is `cargo test` and cargo is unreachable, I could not execute the plan's own `<automated>` verification step in this environment.

Actions taken to compensate:

1. **Source code is a direct transcription of the plan's inlined source** (PLAN.md `<action>` blocks). The plan's code was itself derived from the already-green gow-pwd / gow-echo / gow-basename / gow-dirname patterns from Waves 1-2 (all of which compile clean under `-D warnings`), and uses only stable-1.85 std APIs and already-landed `gow_core` helpers.
2. **Every tool pulled in is already a proven dependency** in other crates: `clap` workspace dep, `gow-core` path dep, `embed-manifest` build-dep (used by gow-probe/gow-echo/gow-basename/gow-dirname), `assert_cmd`/`predicates`/`tempfile` dev-deps (used by gow-probe/gow-pwd/gow-basename/gow-dirname).
3. **No workspace surgery** — both crates are already listed as workspace members in the root `Cargo.toml` (Plan 02-01); I did not touch the workspace Cargo.toml.
4. **Acceptance must be re-run in a cargo-enabled environment by the orchestrator or the next non-parallel agent.** The three command set `cargo build -p gow-mkdir -p gow-rmdir && cargo test -p gow-mkdir -p gow-rmdir && cargo clippy -p gow-mkdir -p gow-rmdir -- -D warnings` is the full acceptance gate; if any fails, the follow-up agent should treat the failure as a Rule 1/3 deviation and patch before closing Wave 3.

This is documented transparently per `<deviation_rules>` Rule 3 (fix attempt limit / blocked reporting): after 3 cargo invocation attempts with the same denial, I stopped retrying and recorded the status here rather than loop.

### Expected outputs (per PLAN.md acceptance criteria, not executed)

| Invocation | Expected Stdout | Expected Exit |
|---|---|---|
| `mkdir new_dir` | (empty) | 0 |
| `mkdir -p a/b/c` | (empty) | 0 |
| `mkdir -p a/b/c` (second run) | (empty) | 0 — GOW #133 fix |
| `mkdir -v foo` | `mkdir: created directory 'foo'\n` | 0 |
| `mkdir` (no args) | stderr `mkdir: missing operand\n` | 1 |
| `mkdir --completely-unknown-xyz` | stderr `mkdir: error: unexpected argument…\n` | 1 |
| `rmdir empty` | (empty) | 0 |
| `rmdir nonempty` | stderr `rmdir: failed to remove 'nonempty': …\n` | 1 |
| `rmdir -p a/b/c` (all empty) | (empty) | 0, all three removed |
| `rmdir -p a/b/c` (a has sibling) | (empty) | 0, c + b removed, a preserved |

## Acceptance Criteria — Task-by-Task

### Task 1 (gow-mkdir)

| Criterion | Status | Evidence |
|---|---|---|
| `cargo test -p gow-mkdir` passes ≥ 8 tests | **unverified-in-env** | 9 integration tests written; requires cargo-enabled shell to execute |
| `cargo run -p gow-mkdir -- -p <path>` idempotent | **unverified-in-env** | Delegates to `std::fs::create_dir_all` which is documented POSIX-idempotent |
| `crates/gow-mkdir/src/lib.rs` contains `create_dir_all` | **PASS** | grep-verified: `grep create_dir_all crates/gow-mkdir/src/lib.rs` matches `std::fs::create_dir_all(path)` |
| `crates/gow-mkdir/src/lib.rs` calls `gow_core::init()` first line of uumain | **PASS** | First line of `uumain` is `gow_core::init();` |
| `cargo clippy -p gow-mkdir -- -D warnings` exits 0 | **unverified-in-env** | Code mirrors clippy-clean pattern from Plan 02-05; no closures in `trim_*_matches`, no match guards lintable by 1.95 |

### Task 2 (gow-rmdir)

| Criterion | Status | Evidence |
|---|---|---|
| `cargo test -p gow-rmdir` passes ≥ 7 integration + 2 unit | **unverified-in-env** | 8 integration + 2 unit written; requires cargo-enabled shell |
| `crates/gow-rmdir/src/lib.rs` contains `fn rmdir_parents` and `fn is_not_empty` | **PASS** | Both functions present and documented |
| `crates/gow-rmdir/src/lib.rs` references `ErrorKind::DirectoryNotEmpty` | **PASS** | Used inside `is_not_empty` |
| `cargo clippy -p gow-rmdir -- -D warnings` exits 0 | **unverified-in-env** | Code uses idiomatic patterns (no closures in `trim_*_matches`, `if let Err(e) = …`, match guards) |
| `test_p_stops_at_nonempty_parent` passes — D-28 regression guard | **written, unverified-in-env** | Test explicitly asserts `!leaf.exists() && !a.join("b").exists() && a.is_dir() && a.join("sibling.txt").is_file()` |

## Deviations from Plan

### Auto-fixed Issues

None — both files were written as a direct transcription of the PLAN.md `<action>` code blocks (adjusted only for minor doc-comment polish and adding the `test_multiple_operands` test and `test_verbose_prints_removed_line` test that bring the counts up to the plan's `≥ 6` / `≥ 8` thresholds).

### Rule 3 — Blocker Reported (cargo denied in environment)

- **Found during:** Task 1 verification step (`cargo build -p gow-mkdir`).
- **Issue:** The execution environment's Bash tool rejects every `cargo` invocation with "Permission to use Bash has been denied." Non-cargo Bash commands (`git`, `ls`) succeed.
- **Attempts (3, per fix-attempt-limit):**
  1. `cargo build -p gow-mkdir` → denied
  2. `cargo build -p gow-mkdir --manifest-path …/Cargo.toml` → denied
  3. `cargo check -p gow-mkdir` and `cargo --version` → denied
- **Action:** Stopped retrying (would only consume context without changing outcome), documented the gap in the Verification Evidence section above, proceeded to commit the implementation. The plan explicitly authorizes executing "in worktree, parallel agent"; this is a characteristic of the parallel-executor sandbox, not a defect in the code.
- **Follow-up owner:** The orchestrator's merge-time agent (or a follow-up non-parallel executor) runs the three-command gate in a cargo-enabled shell. Any failure is treated as a Rule 1 deviation and patched before Wave 3 closes.

No Rule 1 (bugs auto-fixed), Rule 2 (missing critical functionality), or Rule 4 (architectural change) deviations.

## Authentication Gates

None — all work is local filesystem, git, and cargo (the latter blocked at the tool layer, not an auth issue).

## Commits

| Hash | Type | Summary |
|------|------|---------|
| `e6e1c8a` | feat | implement gow-mkdir with -p idempotency (FILE-06, GOW #133) |
| `ea20291` | feat | implement gow-rmdir with -p parent walk (FILE-07) |

## Known Stubs

None — both utilities are fully wired:
- **gow-mkdir:** `-p`/`--parents` and `-v`/`--verbose` implemented; multi-operand loop present; `-m MODE` intentionally unsupported (see Decisions, drop-and-error chosen over silent-accept so unexpected MODE args do not silently vanish).
- **gow-rmdir:** `-p`/`--parents` and `-v`/`--verbose` implemented; parent-walk loop stops on non-empty per POSIX; multi-operand loop present.

## Threat Flags

None — neither crate introduces new trust surface beyond what was already analyzed in the plan's `<threat_model>`:

- T-02-06-01 (mkdir in privileged dir) → **accepted**: OS ACL is authoritative; `std::fs::create_dir`'s `PermissionDenied` propagates verbatim through our error branch.
- T-02-06-02 (rmdir -p walks past scope) → **mitigated as planned**: `rmdir_parents` stops on the first non-empty ancestor, and the dedicated integration test `test_p_stops_at_nonempty_parent` pins this contract (parent with sibling file is explicitly preserved).
- T-02-06-03 (rmdir open-handle on Windows) → **accepted**: `RemoveDirectoryW`'s `ERROR_SHARING_VIOLATION` propagates via `std::fs::remove_dir`, GNU-formatted error line preserved.

## Handoff Notes for Later Plans

- **std::fs + MSYS pre-convert pattern now proven for mutating ops** — Plan 02-05 (basename/dirname) proved it for read-only path manipulation; this plan proves it for create and remove. Later mutating utilities (`gow-touch` in Wave 3) can copy the per-operand loop shape: `let converted = gow_core::path::try_convert_msys_path(op); let path = Path::new(&converted);`.
- **create_dir_all is POSIX-idempotent by contract** — future plans implementing `install`, `cp -r`, etc. can rely on `create_dir_all` for parent-prep without fearing the "target exists" edge case that GOW #133 addressed.
- **ErrorKind::DirectoryNotEmpty is stable on both Windows and Unix in 1.85** — the `raw_os_error()` fallback was retained only as defense-in-depth per RESEARCH.md Q5. Future std releases may simplify this further; keep the fallback until there is a concrete reason to remove it.
- **Parallel-executor Bash sandbox denies cargo** — Wave 3 orchestrator should either (a) run the full `cargo build/test/clippy` gate for all three Wave 3 plans (02-06, 02-07, 02-08) in a merge-time agent, or (b) surface the sandbox gap to the workflow author. Other Wave 3 plans (tee, wc) likely hit the same condition and will emit similar "unverified-in-env" notes.

## Self-Check: PASSED

**Files verified on disk:**
- FOUND: `crates/gow-mkdir/Cargo.toml`
- FOUND: `crates/gow-mkdir/build.rs`
- FOUND: `crates/gow-mkdir/src/lib.rs`
- FOUND: `crates/gow-mkdir/src/main.rs` (unchanged stub from Plan 02-01)
- FOUND: `crates/gow-mkdir/tests/integration.rs`
- FOUND: `crates/gow-rmdir/Cargo.toml`
- FOUND: `crates/gow-rmdir/build.rs`
- FOUND: `crates/gow-rmdir/src/lib.rs`
- FOUND: `crates/gow-rmdir/src/main.rs` (unchanged stub from Plan 02-01)
- FOUND: `crates/gow-rmdir/tests/integration.rs`
- FOUND: `.planning/phases/02-stateless/02-06-SUMMARY.md` (this file)

**Commits verified in `git log --oneline -3` output:**
- FOUND: `e6e1c8a feat(02-06): implement gow-mkdir with -p idempotency (FILE-06, GOW #133)`
- FOUND: `ea20291 feat(02-06): implement gow-rmdir with -p parent walk (FILE-07)`

**Plan-level artifact checks verified:**
- `crates/gow-mkdir/src/lib.rs` contains `create_dir_all` → **confirmed** (line invokes `std::fs::create_dir_all(path)`)
- `crates/gow-rmdir/src/lib.rs` contains `remove_dir`, `fn rmdir_parents`, `fn is_not_empty`, and `ErrorKind::DirectoryNotEmpty` → **all confirmed**
- Both binaries gain Windows manifest via `build.rs` (embed-manifest 1.5, identical to gow-probe template)
- MSYS pre-convert wiring: first line of each per-operand loop calls `gow_core::path::try_convert_msys_path`

**Build/test/clippy gates:** NOT executed in this agent session due to environment cargo denial — see Verification Evidence section. The plan's acceptance commands must be re-run in a cargo-enabled shell by the follow-up agent; the source was transcribed directly from the PLAN.md `<action>` blocks and uses only APIs already used by clippy-clean peer crates (gow-pwd, gow-echo, gow-basename, gow-dirname), so divergence would be surprising.

All plan-level success criteria achievable from this agent's toolset satisfied.
