---
phase: 02-stateless
plan: 01
subsystem: infra
tags: [cargo, workspace, stub-crates, scaffolding, phase-2-setup]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: gow-core public API (init, args::parse_gnu, path::try_convert_msys_path, error::GowError) — stubs link against it
provides:
  - Workspace Cargo.toml lists 16 members (gow-core + gow-probe + 14 Phase 2 utility crates)
  - snapbox 1.2, bstr 1, filetime 0.2 pinned in [workspace.dependencies] — ready for inheritance by per-utility plans
  - 14 compile-clean stub crates (crates/gow-{echo,pwd,env,tee,basename,dirname,yes,true,false,mkdir,rmdir,touch,wc,which}) with Cargo.toml + src/lib.rs + src/main.rs
  - gow-true and gow-false already final (D-22: trivial 0/1 exit) — no later plan needs to touch uumain for these two
  - .claude/ added to .gitignore to keep agent-local tooling out of the repo
affects: [02-02, 02-03, 02-04, 02-05, 02-06, 02-07, 02-08, 02-09, 02-10, 02-11]

# Tech tracking
tech-stack:
  added:
    - snapbox 1.2 (workspace.dependencies — snapshot testing per D-30a)
    - bstr 1 (workspace.dependencies — byte-safe iteration for wc per D-17)
    - filetime 0.2 (workspace.dependencies — touch timestamps per Q2)
  patterns:
    - "uumain signature: pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32"
    - "Thin main.rs wrapper: std::process::exit(uu_{name}::uumain(std::env::args_os()))"
    - "[[bin]] name drops gow- prefix (echo, pwd, env, ...) while [lib] name prefixes uu_ (uu_echo, uu_pwd, uu_env, ...)"
    - "Trivial utilities (gow-true, gow-false) skip clap/anyhow/thiserror — deps list is just gow-core"
    - "Stubs intentionally omit build.rs — each utility's Wave 2/3/4 plan adds embed-manifest build.rs alongside its real uumain body in a single commit"

key-files:
  created:
    - .planning/phases/02-stateless/02-01-SUMMARY.md
    - crates/gow-echo/Cargo.toml
    - crates/gow-echo/src/lib.rs
    - crates/gow-echo/src/main.rs
    - crates/gow-pwd/Cargo.toml
    - crates/gow-pwd/src/lib.rs
    - crates/gow-pwd/src/main.rs
    - crates/gow-env/Cargo.toml
    - crates/gow-env/src/lib.rs
    - crates/gow-env/src/main.rs
    - crates/gow-tee/Cargo.toml
    - crates/gow-tee/src/lib.rs
    - crates/gow-tee/src/main.rs
    - crates/gow-basename/Cargo.toml
    - crates/gow-basename/src/lib.rs
    - crates/gow-basename/src/main.rs
    - crates/gow-dirname/Cargo.toml
    - crates/gow-dirname/src/lib.rs
    - crates/gow-dirname/src/main.rs
    - crates/gow-yes/Cargo.toml
    - crates/gow-yes/src/lib.rs
    - crates/gow-yes/src/main.rs
    - crates/gow-true/Cargo.toml
    - crates/gow-true/src/lib.rs
    - crates/gow-true/src/main.rs
    - crates/gow-false/Cargo.toml
    - crates/gow-false/src/lib.rs
    - crates/gow-false/src/main.rs
    - crates/gow-mkdir/Cargo.toml
    - crates/gow-mkdir/src/lib.rs
    - crates/gow-mkdir/src/main.rs
    - crates/gow-rmdir/Cargo.toml
    - crates/gow-rmdir/src/lib.rs
    - crates/gow-rmdir/src/main.rs
    - crates/gow-touch/Cargo.toml
    - crates/gow-touch/src/lib.rs
    - crates/gow-touch/src/main.rs
    - crates/gow-wc/Cargo.toml
    - crates/gow-wc/src/lib.rs
    - crates/gow-wc/src/main.rs
    - crates/gow-which/Cargo.toml
    - crates/gow-which/src/lib.rs
    - crates/gow-which/src/main.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - .gitignore

decisions:
  - "Ignore .claude/ locally — this directory holds Claude Code agent settings + worktree scaffolding and is environment-specific, not project artifact"
  - "Keep gow-true / gow-false final at scaffold time (D-22 says uumain is 0/1; no need for a stub that throws away args then gets replaced later)"

metrics:
  duration: "~4 minutes"
  completed: "2026-04-21"
  tasks_completed: 2
  files_created: 42
  files_modified: 3
  commits: 2
---

# Phase 2 Plan 01: Workspace Prep — 14 Stub Crates Summary

**One-liner:** Workspace Cargo.toml expanded from 2 to 16 members, 3 Phase 2 workspace deps pinned (snapbox/bstr/filetime), and 14 compile-clean stub utility crates scaffolded so Waves 2/3/4 can be filled in parallel without touching the workspace root.

## Objective

Phase 2 has 11 utility plans that will be executed across 3 waves. Each wave can only run in parallel if (a) the workspace members list already includes every utility's directory, and (b) each such directory has at least a minimal `Cargo.toml` so `cargo metadata` resolves. This plan solves both prerequisites in a single commit pair — the root edit + the 42 stub files — so no wave 2+ plan ever needs to modify `Cargo.toml` at the workspace root.

The secondary effect: `cargo build --workspace` stays green from this point forward, meaning any CI run between now and Phase 2 completion will detect per-utility regressions immediately.

## What Was Built

### Task 1 — Workspace root manifest edit (commit `6911942`)

- `members` array expanded from `["crates/gow-core", "crates/gow-probe"]` to a 16-entry block listing gow-core, gow-probe, and the 14 Phase 2 crate paths in D-16 order. Inline comment `# Phase 2 — stateless utilities (D-16)` marks the phase boundary inside the TOML for future readers.
- `[workspace.dependencies]` gained three new entries below the existing `tempfile = "3"` line, each pinned exactly per D-20a:
  - `snapbox = "1.2"` — snapshot testing (D-30a)
  - `bstr = "1"` — byte-safe iteration for wc (D-17)
  - `filetime = "0.2"` — file timestamps for touch (Q2)
- `[workspace.package]` and `[profile.release]` untouched (acceptance criterion).

### Task 2 — 14 stub utility crates (commit `36053fd`)

42 files created — one `Cargo.toml`, one `src/lib.rs`, one `src/main.rs` per crate.

| Crate | Bin name | Lib name | Deps beyond gow-core | Stub exit | Stub stderr |
|-------|----------|----------|----------------------|-----------|-------------|
| gow-echo | echo | uu_echo | clap, anyhow, thiserror | 1 | `echo: not yet implemented` |
| gow-pwd | pwd | uu_pwd | clap, anyhow, thiserror | 1 | `pwd: not yet implemented` |
| gow-env | env | uu_env | clap, anyhow, thiserror | 1 | `env: not yet implemented` |
| gow-tee | tee | uu_tee | clap, anyhow, thiserror | 1 | `tee: not yet implemented` |
| gow-basename | basename | uu_basename | clap, anyhow, thiserror | 1 | `basename: not yet implemented` |
| gow-dirname | dirname | uu_dirname | clap, anyhow, thiserror | 1 | `dirname: not yet implemented` |
| gow-yes | yes | uu_yes | clap, anyhow, thiserror | 1 | `yes: not yet implemented` |
| gow-mkdir | mkdir | uu_mkdir | clap, anyhow, thiserror | 1 | `mkdir: not yet implemented` |
| gow-rmdir | rmdir | uu_rmdir | clap, anyhow, thiserror | 1 | `rmdir: not yet implemented` |
| gow-touch | touch | uu_touch | clap, anyhow, thiserror | 1 | `touch: not yet implemented` |
| gow-wc | wc | uu_wc | clap, anyhow, thiserror | 1 | `wc: not yet implemented` |
| gow-which | which | uu_which | clap, anyhow, thiserror | 1 | `which: not yet implemented` |
| gow-true | true | uu_true | *(none — D-22 trimmed)* | 0 | *(silent)* |
| gow-false | false | uu_false | *(none — D-22 trimmed)* | 1 | *(silent)* |

Line counts for representative files:

| File | Lines |
|------|-------|
| gow-echo/Cargo.toml (full template) | 22 |
| gow-true/Cargo.toml (trimmed, D-22) | 19 |
| gow-false/Cargo.toml (trimmed, D-22) | 19 |
| gow-echo/src/lib.rs (full stub with init + error) | 10 |
| gow-true/src/lib.rs (returns 0) | 7 |
| gow-false/src/lib.rs (returns 1) | 7 |
| every src/main.rs (thin wrapper) | 3 |

## Verification Evidence

```
$ cargo build --workspace
   Compiling gow-core v0.1.0 (...)
   Compiling gow-probe v0.1.0 (...)
   [14 Phase 2 crates Compiling ...]
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.61s
  (0 warnings, 0 errors)

$ cargo clippy --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.32s
  (clean)

$ cargo test --workspace
  test result: ok. 34 passed; 0 failed    # gow-core unit
  test result: ok. 9 passed; 0 failed     # gow-probe integration
  test result: ok. 3 passed; 0 failed     # gow-core doctests
  (14 stub crates: 0 tests each — expected, no regressions)

$ cargo run -p gow-true;  echo $?   -> exit 0
$ cargo run -p gow-false; echo $?   -> exit 1
$ cargo run -p gow-echo 2>&1 ; echo $?
  echo: not yet implemented
  exit 1

$ cargo metadata --format-version 1 --no-deps | grep -c '"name":"gow-'
  16   # gow-core + gow-probe + 14 Phase 2 utility crates
```

## Acceptance Criteria — Task-by-Task

### Task 1
- [x] `grep -c '"crates/gow-' Cargo.toml` → 16
- [x] `grep -E '^snapbox = "1\.2"$'` → 1 match
- [x] `grep -E '^bstr = "1"$'` → 1 match
- [x] `grep -E '^filetime = "0\.2"$'` → 1 match
- [x] `clap = { version = "4.6", features = ["derive"] }` line unchanged
- [x] `[workspace.package]` block byte-unchanged
- [x] `[profile.release]` block byte-unchanged

### Task 2
- [x] 42 stub files (14 × 3) created at exact expected paths
- [x] `cargo build --workspace` exits 0 with zero warnings and zero errors
- [x] `cargo run -p gow-true` exits 0
- [x] `cargo run -p gow-false` exits 1
- [x] `cargo run -p gow-echo` exits 1 with stderr `echo: not yet implemented`
- [x] `cargo metadata --format-version 1 --no-deps` lists exactly 16 workspace members
- [x] Each stub `Cargo.toml` inherits `version.workspace = true` and `edition.workspace = true`
- [x] `crates/gow-true/Cargo.toml` has no clap/anyhow/thiserror
- [x] `crates/gow-false/Cargo.toml` has no clap/anyhow/thiserror
- [x] Every `src/main.rs` is the 3-line wrapper `std::process::exit(uu_{name}::uumain(std::env::args_os()));`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Hygiene] Added `.claude/` to .gitignore**
- **Found during:** Task 2 post-commit status check.
- **Issue:** After Task 2's file creation, `git status --short` listed `.claude/` (containing `settings.local.json` + agent worktree scratch) as untracked. This is agent-local tooling state — not project work, not gitignored, and it would show up in every future `git status` run across the Phase 2 plan sequence.
- **Fix:** Appended `.claude/` entry to `.gitignore` (with a one-line comment explaining it's Claude Code local tooling). This keeps working-tree status clean for the remaining Phase 2 plans.
- **Files modified:** `.gitignore` (+3 lines)
- **Commit:** `36053fd` (bundled with Task 2 because both touch working-tree hygiene)

No Rule 1 bugs, no Rule 3 blockers, no Rule 4 architectural decisions required.

## Authentication Gates

None — all work is local filesystem + cargo.

## Commits

| Hash | Type | Summary |
|------|------|---------|
| `6911942` | chore | Extend workspace with 14 Phase 2 members + snapbox/bstr/filetime |
| `36053fd` | feat | Scaffold 14 stub utility crates for Phase 2 |

## Handoff Notes for Wave 2/3/4 Plans

- **DO NOT edit the root `Cargo.toml`** when filling in a utility — the members list and workspace.dependencies are already complete. If a utility needs a crate-specific dep (e.g., `jiff` for touch, `embed-manifest` for any bin crate's build-dep), add it to `crates/gow-{name}/Cargo.toml` `[dependencies]` or `[build-dependencies]` only.
- **Stub uumain body is the first thing your plan should rewrite** — the current `eprintln!("{name}: not yet implemented"); 1` is explicitly a placeholder. Replace `fn uumain` entirely; do not try to "extend" it.
- **build.rs is NOT present on any Phase 2 stub** — this is intentional. When your plan adds the real uumain, the same commit should also add `crates/gow-{name}/build.rs` by copying `crates/gow-probe/build.rs` verbatim (per D-16c). Update the `[build-dependencies]` section to add `embed-manifest = "1.5"`.
- **`gow-true` / `gow-false` need no further uumain work** — per D-22 the body is already final (returns 0 / 1). Plan 02-02 should only add integration tests + build.rs for these two.
- **Workspace-level inheritance** — when a utility's Cargo.toml needs snapbox/bstr/filetime, use `{ workspace = true }` (they're pinned at the workspace root). Only jiff / parse_datetime / dunce style per-utility deps go with explicit version strings.
- **Tests directory** — no `tests/` subdir exists on any stub yet. When your plan adds integration tests, create `crates/gow-{name}/tests/integration.rs` and ensure `[dev-dependencies]` in the crate's Cargo.toml gains `assert_cmd`, `predicates`, `tempfile`, and (where needed) `snapbox`, each with `{ workspace = true }`.

## Self-Check: PASSED

**Files verified on disk:**
- FOUND: `D:\workspace\gow-rust\Cargo.toml` (16 gow- members; snapbox/bstr/filetime pinned)
- FOUND: all 14 × 3 = 42 stub files under `D:\workspace\gow-rust\crates\gow-{14 names}\`
- FOUND: `D:\workspace\gow-rust\.gitignore` (with `.claude/` entry)
- FOUND: `D:\workspace\gow-rust\.planning\phases\02-stateless\02-01-SUMMARY.md` (this file)

**Commits verified in git log:**
- FOUND: `6911942` chore(02-01): extend workspace with 14 Phase 2 members + snapbox/bstr/filetime
- FOUND: `36053fd` feat(02-01): scaffold 14 stub utility crates for Phase 2

**Build/test gates verified in session output:**
- `cargo build --workspace` → exit 0, 0 warnings, 0 errors
- `cargo clippy --workspace -- -D warnings` → exit 0
- `cargo test --workspace` → 34 + 9 + 3 passing; 0 failed; no regressions from Phase 1
- `cargo run -p gow-true` → exit 0 (final behavior per D-22)
- `cargo run -p gow-false` → exit 1 (final behavior per D-22)
- `cargo run -p gow-echo` → stub message + exit 1 (as designed)
- `cargo metadata --format-version 1 --no-deps | grep -c '"name":"gow-'` → 16

All plan-level success criteria satisfied. Ready to hand off to Phase 2 Wave 2 plans (02-02 through 02-11).
