---
phase: 03-filesystem
plan: 01
subsystem: workspace-prep + gow-core-extension
tags: [phase3, workspace-prep, gow-core-extension, wave-0]
dependency_graph:
  requires:
    - "crates/gow-core (Phase 1 foundation — args, path, encoding, color)"
    - "crates/gow-touch (Phase 2 template — build.rs + Cargo.toml shape)"
  provides:
    - "gow_core::fs::atomic_rewrite — same-directory temp+rename for sed/dos2unix"
    - "gow_core::fs::create_link / LinkKind / LinkOutcome — D-36 junction fallback"
    - "gow_core::fs::is_hidden — D-34 dot-prefix OR FILE_ATTRIBUTE_HIDDEN"
    - "gow_core::fs::is_readonly — ls -l permission bit"
    - "gow_core::fs::has_executable_extension — D-35 hardcoded set"
    - "gow_core::fs::clear_readonly — D-45 rm -f prerequisite"
    - "gow_core::fs::is_drive_root — D-42 rm / safety"
    - "crates/gow-{cat,ls,cp,mv,rm,ln,chmod,head,tail,dos2unix,unix2dos} — 11 stub crates ready for Wave 1-5 parallel execution"
    - "workspace.dependencies: walkdir 2.5, notify 8.2, terminal_size 0.4, junction 1.4"
  affects:
    - "Every Phase 3 Wave 1-5 plan (03-02 through 03-12) — unblocked"
tech_stack:
  added:
    - "walkdir 2.5 (workspace.dependencies) — recursive traversal for cp, rm, ls, chmod"
    - "notify 8.2 (workspace.dependencies) — tail -f file watcher via ReadDirectoryChangesW"
    - "terminal_size 0.4 (workspace.dependencies) — ls column layout"
    - "junction 1.4 (workspace.dependencies + gow-core windows-cfg dep) — D-36 directory symlink fallback"
    - "tempfile (promoted from dev-dep to runtime dep in gow-core) — atomic_rewrite tempfile placement"
  patterns:
    - "gow-touch Cargo.toml shape replicated across 11 new stub crates"
    - "gow-touch build.rs copied byte-identical (cmp == 0) to 11 new crates"
    - "Stub uumain pattern (gow_core::init + eprintln + return 1) matches Phase 2 Plan 02-01"
    - "ERROR_PRIVILEGE_NOT_HELD (raw_os_error == Some(1314)) branch in create_link — new pattern first applied here"
key_files:
  created:
    - path: "crates/gow-cat/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU cat stub — FILE-01 placeholder"
    - path: "crates/gow-ls/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU ls stub — FILE-02 placeholder (walkdir + terminal_size + termcolor + bstr declared)"
    - path: "crates/gow-cp/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU cp stub — FILE-03 placeholder (walkdir + filetime declared)"
    - path: "crates/gow-mv/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU mv stub — FILE-04 placeholder (filetime declared)"
    - path: "crates/gow-rm/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU rm stub — FILE-05 placeholder (walkdir declared)"
    - path: "crates/gow-ln/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU ln stub — FILE-09 placeholder (junction as windows-cfg target dep)"
    - path: "crates/gow-chmod/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU chmod stub — FILE-10 placeholder (walkdir declared)"
    - path: "crates/gow-head/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU head stub — TEXT-01 placeholder (bstr declared)"
    - path: "crates/gow-tail/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "GNU tail stub — TEXT-02 placeholder (bstr + notify declared)"
    - path: "crates/gow-dos2unix/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "dos2unix stub — CONV-01 placeholder (bstr + filetime declared)"
    - path: "crates/gow-unix2dos/Cargo.toml + build.rs + src/main.rs + src/lib.rs"
      purpose: "unix2dos stub — CONV-02 placeholder (bstr + filetime declared)"
  modified:
    - path: "Cargo.toml (workspace root)"
      change: "Added 11 new crate members + 4 new workspace.dependencies"
    - path: "crates/gow-core/Cargo.toml"
      change: "Promoted tempfile to runtime dep; added junction as [target.'cfg(windows)'.dependencies]"
    - path: "crates/gow-core/src/fs.rs"
      change: "Added 7 helpers (atomic_rewrite, create_link+LinkKind+LinkOutcome, is_hidden, is_readonly, has_executable_extension, clear_readonly, is_drive_root) + 12 new unit tests"
    - path: "Cargo.lock"
      change: "Regenerated with new workspace deps graph"
decisions:
  - "Stub crates declare per-crate extras (bstr, walkdir, filetime, notify, etc.) up-front — Wave 1-5 plans can run in parallel worktrees without touching workspace Cargo.toml (deferred dep additions would race on root Cargo.toml)"
  - "build.rs copied byte-identical (via cp) from gow-touch — any drift would have to be tracked per-crate; one source of truth for manifest-embedding is simpler than diff tracking"
  - "clippy::permissions_set_readonly_false allow attribute applied at function-level on clear_readonly + test-function-level on test_is_readonly_after_set — Windows RO bit is a single-bit attribute that the Unix lint warns about doesn't apply"
metrics:
  duration_minutes: 10
  completed_date: "2026-04-21"
  tasks_completed: 3
  files_created: 44
  files_modified: 4
  tests_added: 24
  test_count_before: 265
  test_count_after: 289
---

# Phase 03 Plan 01: Workspace Prep + gow-core::fs Extension + 11 Stub Crates Summary

Phase 3 Wave 0 — single blocking plan that unblocks every Wave 1-5 plan. Adds 4 workspace dependencies (walkdir, notify, terminal_size, junction), extends `gow_core::fs` with 7 filesystem helpers covering D-47/D-36/D-38/D-34/D-35/D-45/D-42, and scaffolds 11 stub crates (cat/ls/cp/mv/rm/ln/chmod/head/tail/dos2unix/unix2dos) using the gow-touch template verbatim.

## What Was Delivered

### Task 1 — Workspace Cargo.toml + gow-core/Cargo.toml (commit 5b33238)

- **`Cargo.toml [workspace.dependencies]`** +4 entries:
  - `walkdir = "2.5"` — recursive traversal for cp, rm, ls, chmod
  - `notify = "8.2"` — tail -f watcher (ReadDirectoryChangesW backend)
  - `terminal_size = "0.4"` — ls column layout width detection
  - `junction = "1.4"` — D-36 directory-symlink fallback (closing RESEARCH.md gap)
- **`Cargo.toml [workspace.members]`** +11 entries, comment-annotated:
  `gow-cat, gow-ls, gow-cp, gow-mv, gow-rm, gow-ln, gow-chmod, gow-head, gow-tail, gow-dos2unix, gow-unix2dos`
- **`crates/gow-core/Cargo.toml`** — `tempfile = { workspace = true }` promoted from dev-dep to runtime dep (for atomic_rewrite), added `[target.'cfg(windows)'.dependencies] junction = { workspace = true }` for the D-36 fallback.

### Task 2 — gow_core::fs extensions + tests (commit d3a2205)

7 new public items in `crates/gow-core/src/fs.rs`:

| Helper | Signature | Purpose |
|--------|-----------|---------|
| `atomic_rewrite` | `fn atomic_rewrite<F>(path: &Path, transform: F) -> Result<(), GowError> where F: FnOnce(&[u8]) -> Result<Vec<u8>, GowError>` | Same-directory NamedTempFile + persist (MoveFileExW on Windows) — D-47 |
| `LinkKind` | `enum LinkKind { Hard, Symbolic }` | Input selector for create_link |
| `LinkOutcome` | `enum LinkOutcome { Symlink, Junction, Hardlink }` | Output so caller can emit D-36 fallback warning |
| `create_link` | `fn create_link(target: &Path, link_path: &Path, kind: LinkKind) -> io::Result<LinkOutcome>` | Handles D-36 (ERROR_PRIVILEGE_NOT_HELD 1314 → junction fallback) + D-38 (hard link same-volume) |
| `is_hidden` | `fn is_hidden(path: &Path) -> bool` | D-34 union: dot-prefix OR FILE_ATTRIBUTE_HIDDEN (0x2) |
| `is_readonly` | `fn is_readonly(md: &std::fs::Metadata) -> bool` | Permissions().readonly() wrapper for ls -l |
| `has_executable_extension` | `fn has_executable_extension(path: &Path) -> bool` | Matches `.exe .cmd .bat .ps1 .com` case-insensitive (D-35, no PATHEXT consult) |
| `clear_readonly` | `fn clear_readonly(path: &Path) -> std::io::Result<()>` | Windows-only clears RO bit; no-op elsewhere (D-45) |
| `is_drive_root` | `fn is_drive_root(path: &Path) -> bool` | Detects `C:\`, `C:/`, `C:`, and UNC share roots — D-42 rm safety |

12 new unit tests (in addition to 8 pre-existing fs::tests): atomic_rewrite roundtrip/transform/error-preserves-original, hard_link same-volume, symlink file privilege-skip, symlink dir D-36 fallback (accepts either Symlink or Junction outcome), is_hidden dot-prefix + not-hidden, is_readonly default + after_set, has_executable_extension matrix, clear_readonly (Windows only), is_drive_root variants (C:\\, C:/, C:, UNC share, sub-paths, server-only).

`GowError::Io { path, source }` and `GowError::Custom(String)` variants already existed in `crates/gow-core/src/error.rs` — no error.rs changes needed.

### Task 3 — 11 stub crates (commit 3918182)

Each crate follows the Phase 2 Plan 02-01 stub shape:

- `Cargo.toml` — workspace-inherited package metadata, `[[bin]]` + `[lib]` = `uu_{name}`, gow-core path dep + clap/anyhow/thiserror workspace + per-crate extras + embed-manifest 1.5 build-dep + test dev-deps
- `build.rs` — byte-identical (`cmp` returns 0) to `crates/gow-touch/build.rs`; embeds `Gow.Rust` manifest with UTF-8 active code page + long path aware
- `src/main.rs` — 3-line shim `std::process::exit(uu_{name}::uumain(std::env::args_os()))`
- `src/lib.rs` — stub `uumain` calls `gow_core::init()`, `eprintln!("{name}: not yet implemented")`, returns 1; one smoke test `stub_returns_one`

**Per-crate declared extras (already in Cargo.toml so Wave 1-5 plans don't touch workspace root):**

| Crate | Extras | Requirement |
|-------|--------|-------------|
| gow-cat | bstr | FILE-01 |
| gow-ls | walkdir + terminal_size + termcolor + bstr | FILE-02 |
| gow-cp | walkdir + filetime | FILE-03 |
| gow-mv | filetime | FILE-04 |
| gow-rm | walkdir | FILE-05 |
| gow-ln | junction (target.'cfg(windows)') | FILE-09 |
| gow-chmod | walkdir | FILE-10 |
| gow-head | bstr | TEXT-01 |
| gow-tail | bstr + notify | TEXT-02 |
| gow-dos2unix | bstr + filetime | CONV-01 |
| gow-unix2dos | bstr + filetime | CONV-02 |

## Verification Results

```
cargo build --workspace          -> OK (26 crates compiled, 11 new exe binaries in target/*/debug/)
cargo test --workspace           -> 289 passed, 0 failed (was 265; +24: 12 gow-core::fs tests + 11 stub smoke tests + 1 net)
cargo clippy --workspace --all-targets -- -D warnings -> OK (after Rule 1 deviation below)

./target/x86_64-pc-windows-msvc/debug/cat.exe -> "cat: not yet implemented", exit 1 (spot-check)
```

All 11 binaries confirmed present: `basename.exe, cat.exe, chmod.exe, cp.exe, dirname.exe, dos2unix.exe, echo.exe, env.exe, false.exe, gow-probe.exe, head.exe, ln.exe, ls.exe, mkdir.exe, mv.exe, pwd.exe, rm.exe, rmdir.exe, tail.exe, tee.exe, touch.exe, true.exe, unix2dos.exe, wc.exe, which.exe, yes.exe`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Lint] clippy::permissions_set_readonly_false**

- **Found during:** Task 2 verification (clippy workspace run)
- **Issue:** `clear_readonly` in `crates/gow-core/src/fs.rs:273` and test cleanup at line 530 call `Permissions::set_readonly(false)`, which clippy 1.95 flags because on Unix it makes the file world-writable (0o666 vs prior mode).
- **Fix:** Added `#[allow(clippy::permissions_set_readonly_false)]` at (a) function-level on `clear_readonly` with rationale that the function is no-op on non-Windows targets, (b) function-level on `test_is_readonly_after_set` with rationale that ephemeral tempdir content has no lint-relevant security implication. Inline comments explain the Windows RO-bit semantics.
- **Files modified:** `crates/gow-core/src/fs.rs` (2 lines of attribute + 1 explanatory comment block)
- **Commit:** `d3a2205`

**2. [Rule 1 — Config] build.rs line-ending normalization**

- **Found during:** Task 3 byte-identity verification (cmp between `gow-touch/build.rs` and new crate build.rs)
- **Issue:** Initial Write tool invocations created build.rs files with LF line endings while `gow-touch/build.rs` on disk uses CRLF — `cmp` reported differences at char 32 (first line break).
- **Fix:** Overwrote all 11 build.rs files with `cp -f crates/gow-touch/build.rs crates/gow-{name}/build.rs` so the bytes match exactly. All 11 now pass `cmp` cleanly. Plan explicitly required byte-identical copy.
- **Files modified:** All 11 stub `build.rs` files
- **Commit:** `3918182`

**3. [Rule 3 — Ordering] Task 2 verification requires Task 3 stubs to exist**

- **Found during:** Task 2 `cargo test -p gow-core fs::` invocation
- **Issue:** Task 2 verify command (`cargo test -p gow-core fs::`) failed because `cargo metadata` parses the workspace manifest first, and the Task 1 amendments registered 11 members whose directories did not yet exist. This is a natural ordering artifact of the plan: Task 1 declares members → Task 3 creates them; Task 2's isolated test can't run until both are done.
- **Fix:** Reordered execution internally: Task 1 committed → Task 3 stub files created but not committed → run combined `cargo build --workspace` + `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` which verifies Tasks 2 and 3 together → commit Task 2 (fs.rs + Cargo.lock) → commit Task 3 (stub crates). This preserves per-task atomic commits while working around the plan-level ordering constraint. No plan instruction was bypassed — Task 2's acceptance criteria were met after running the combined verification.
- **Files modified:** None beyond normal task execution
- **Commit:** N/A (process deviation, not code)

### Authentication Gates

None.

## Path Forward for Wave 1-5 Plans

All 11 utility plans (03-02 through 03-12) can now run in parallel worktrees. Each plan:
1. Replaces its crate's `src/lib.rs` stub with real uumain logic
2. Adds `tests/integration.rs`
3. Uses the already-declared per-crate deps (no workspace Cargo.toml edits needed)
4. Calls `gow_core::fs::{atomic_rewrite,create_link,is_hidden,is_readonly,has_executable_extension,clear_readonly,is_drive_root}` as appropriate

The build.rs embed-manifest invocation is already in place, so Wave 1-5 plans do not need to add new build scripts — they just evolve the lib.rs/main.rs contents.

## Self-Check: PASSED

- **Files created** (44 in 11 crate directories): verified via `git log --name-status 3918182 | head -60` + inspection
- **Files modified** (3 source + 1 lockfile):
  - `Cargo.toml` (committed 5b33238): present with walkdir/notify/terminal_size/junction + 11 member entries
  - `crates/gow-core/Cargo.toml` (committed 5b33238): tempfile runtime + junction windows-cfg
  - `crates/gow-core/src/fs.rs` (committed d3a2205): atomic_rewrite + 6 other helpers + 12 tests
  - `Cargo.lock` (committed d3a2205): 4 new deps resolved
- **Commits verified present:**
  - `5b33238` — feat(03-01): add 4 workspace deps + 11 Phase 3 members + gow-core tempfile/junction
  - `d3a2205` — feat(03-01): add 7 filesystem helpers to gow_core::fs
  - `3918182` — feat(03-01): scaffold 11 Phase 3 stub crates using gow-touch template
- **Functional verification:**
  - `cargo build --workspace` → OK
  - `cargo test --workspace` → 289 passed / 0 failed
  - `cargo clippy --workspace --all-targets -- -D warnings` → OK
  - `./target/x86_64-pc-windows-msvc/debug/cat.exe` → prints stub message + exits 1
