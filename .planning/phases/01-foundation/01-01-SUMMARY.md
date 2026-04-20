---
phase: 01-foundation
plan: 01
subsystem: infra
tags: [cargo, workspace, rust-2024, windows-msvc, embed-manifest, crt-static]

# Dependency graph
requires:
  - phase: none
    provides: clean slate — no prior phases
provides:
  - Cargo workspace root with resolver = 3, edition 2024, [workspace.dependencies] for all Phase 1 shared crates
  - .cargo/config.toml pinning x86_64-pc-windows-msvc target with +crt-static (eliminates VCRUNTIME140.dll dependency)
  - gow-core crate skeleton with six module stubs (args, color, encoding, error, fs, path) and pub fn init()
  - gow-core/build.rs as the canonical manifest-embedding template for all utility bin crates (Phase 2+)
  - Verified toolchain: rustc 1.95.0 (MSVC), cargo 1.95.0, all pinned dep versions resolve against the live crates.io registry
affects: [01-02, 01-03, 01-04, 02-stateless, 03-filesystem, 04-text-processing, 05-search-navigation, 06-archive-network]

# Tech tracking
tech-stack:
  added:
    - clap 4.6 (derive)
    - thiserror 2
    - anyhow 1
    - termcolor 1.4
    - windows-sys 0.61 (Win32_System_Console, Win32_Foundation, Win32_Storage_FileSystem)
    - encoding_rs 0.8
    - path-slash 0.2
    - assert_cmd 2
    - predicates 3
    - tempfile 3
    - embed-manifest 1.5 (build-dep)
  patterns:
    - "Cargo workspace: one shared Cargo.lock, resolver 3, edition 2024, [workspace.dependencies] to pin versions"
    - "gow-core as the sole owner of Win32 calls; utility crates never import windows-sys directly"
    - "build.rs template for Windows app manifest (activeCodePage=UTF-8, longPathAware=Enabled) — copied into each bin crate from Phase 2"
    - "Static CRT (+crt-static) globally via .cargo/config.toml so binaries run on machines without the VC++ Redistributable"

key-files:
  created:
    - Cargo.toml
    - .cargo/config.toml
    - .gitignore
    - Cargo.lock
    - crates/gow-core/Cargo.toml
    - crates/gow-core/build.rs
    - crates/gow-core/src/lib.rs
    - crates/gow-core/src/args.rs
    - crates/gow-core/src/color.rs
    - crates/gow-core/src/encoding.rs
    - crates/gow-core/src/error.rs
    - crates/gow-core/src/fs.rs
    - crates/gow-core/src/path.rs
  modified: []

key-decisions:
  - "gow-core remains lib-only in Phase 1; bin target(s) arrive with Plan 01-04 (gow-probe). build.rs gates embed_manifest() on detected bin targets so the same script works unchanged when copied into Phase 2+ utility crates."
  - "Use embed_manifest::manifest::Setting::Enabled for long_path_aware (plan/research text referenced a non-existent LongPathAware::Yes enum — corrected against the embed-manifest 1.5.0 API)."
  - "assert_cmd has no `cargo` feature flag in 2.x (functionality is unconditional); declared plainly as assert_cmd = \"2\"."
  - "Cargo.lock committed — this workspace produces binaries (per FOUND-01 and D-14), so pinning the lockfile in-repo is the Rust idiom."

patterns-established:
  - "Pattern: bin-target-aware build.rs — a shared manifest-embedding script that is safe to keep in lib-only crates and correct when copy-pasted into bin crates"
  - "Pattern: workspace dependency inheritance — each member crate uses `{ workspace = true }` so Phase 2+ utilities inherit versions centrally"
  - "Pattern: module stub + smoke test — Plan 01 establishes the module graph; Plans 02/03 fill in implementations without needing to reshape lib.rs"

requirements-completed: [FOUND-01, FOUND-02, WIN-01, WIN-02, WIN-03]

# Metrics
duration: 6min
completed: 2026-04-20
---

# Phase 1 Plan 01: Cargo Workspace and gow-core Skeleton Summary

**Cargo workspace (resolver 3, edition 2024) with gow-core lib crate, MSVC static-CRT config, and a bin-gated embed-manifest build.rs template — `cargo build --workspace` and `cargo test -p gow-core` are both green.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-20T13:50:05Z
- **Completed:** 2026-04-20T13:56:XX Z
- **Tasks:** 2
- **Files created:** 13

## Accomplishments

- Workspace manifest pins every Phase 1 shared dep at the workspace level; future utility crates inherit versions without drift.
- gow-core compiles cleanly with zero warnings on 1.95.0 MSVC toolchain; clippy `-D warnings` passes.
- build.rs preserves the exact `embed_manifest(...)` call site Phase 2+ utility crates will copy, while safely skipping the call for today's lib-only build.
- Static CRT linkage locked in via `.cargo/config.toml` — release binaries will not depend on `VCRUNTIME140.dll`.
- Cargo.lock checked in (66 packages resolved), reproducing the exact dependency graph for CI and future plans.
- `init_does_not_panic` smoke test proves the module graph links and the stub implementations are callable.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create workspace root Cargo.toml and .cargo/config.toml** — `974a7fe` (feat)
2. **Task 2: Create gow-core crate manifest, build.rs, and lib.rs stub** — `c15706b` (feat)

_Plan metadata commit will land with SUMMARY.md + STATE.md + ROADMAP.md updates._

## Files Created

- `Cargo.toml` — workspace root; `[workspace.dependencies]` for clap/thiserror/anyhow/termcolor/windows-sys/encoding_rs/path-slash/assert_cmd/predicates/tempfile; release profile with strip/lto/opt-z/panic-abort.
- `.cargo/config.toml` — pins `x86_64-pc-windows-msvc` and `+crt-static` rustflag.
- `.gitignore` — excludes `/target`, editor/OS artifacts.
- `Cargo.lock` — 66 packages resolved; committed per workspace-producing-binaries convention.
- `crates/gow-core/Cargo.toml` — workspace-inherited package metadata; embed-manifest 1.5 build-dep; assert_cmd + tempfile dev-deps.
- `crates/gow-core/build.rs` — canonical Windows manifest-embedding template (activeCodePage=UTF-8, long_path_aware=Setting::Enabled); gated on `has_bin_target()` so it is a no-op for lib-only gow-core but works verbatim when copied to bin crates.
- `crates/gow-core/src/lib.rs` — module declarations for six Phase 1 modules and `pub fn init()` wiring `encoding::setup_console_utf8()` + `color::enable_vt_mode()`. Includes `init_does_not_panic` smoke test.
- `crates/gow-core/src/{args,color,encoding,error,fs,path}.rs` — six module stubs with placeholder implementations and coverage-tag comments pointing at the requirement IDs each one will fulfill in Plans 02/03.

## Decisions Made

- **Bin-target-aware build.rs.** Cargo 1.95 rejects `cargo:rustc-link-arg-bins=…` directives for packages without any bin target, which is what embed-manifest emits. Rather than split the manifest template across multiple files or defer it entirely to Plan 04, the build.rs detects `src/main.rs`, `src/bin/`, or a `[[bin]]` table in Cargo.toml before calling `embed_manifest::embed_manifest(...)`. This preserves the literal call signature and the two Setting enum values (`ActiveCodePage::Utf8`, `Setting::Enabled`) exactly where Pitfall 4 requires them — at the top of each binary crate's build script in Phase 2+.
- **Cargo.lock committed.** gow-rust ships binaries. Committing the lockfile matches the upstream Cargo guidance for workspaces that produce executables and guarantees reproducible CI builds.
- **Smoke test, not full coverage.** Plan 01's acceptance criteria only require the scaffold to compile. `init_does_not_panic` gives future plans a failing unit-test signal if a module stub starts panicking on init, without pre-committing to test shapes that Plans 02/03 will design.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed nonexistent `features = ["cargo"]` from assert_cmd workspace dep**
- **Found during:** Task 2 (first `cargo check -p gow-core`)
- **Issue:** Plan and research both declared `assert_cmd = { version = "2", features = ["cargo"] }`. Cargo rejected: "assert_cmd does not have that feature". assert_cmd 2.x exposes its cargo_bin functionality unconditionally — no feature flag exists.
- **Fix:** Changed to `assert_cmd = "2"` in `[workspace.dependencies]`.
- **Files modified:** `Cargo.toml`
- **Verification:** `cargo check -p gow-core` advanced past dependency resolution.
- **Committed in:** `c15706b` (Task 2 commit, since the error surfaced when gow-core began depending on assert_cmd).

**2. [Rule 1 - Bug] Replaced `LongPathAware::Yes` with `Setting::Enabled` in build.rs**
- **Found during:** Task 2 (second `cargo check -p gow-core`)
- **Issue:** Plan and research text referenced `embed_manifest::manifest::LongPathAware::Yes` but this enum does not exist in embed-manifest 1.5.0. The crate uses a shared `Setting::{Enabled, Disabled}` enum for the boolean-style manifest flags.
- **Fix:** Called `.long_path_aware(embed_manifest::manifest::Setting::Enabled)`.
- **Files modified:** `crates/gow-core/build.rs`
- **Verification:** Compile error E0433 went away; `cargo build --workspace` succeeds.
- **Committed in:** `c15706b`.

**3. [Rule 3 - Blocking] Gated `embed_manifest()` call on detected bin target**
- **Found during:** Task 2 (third `cargo check -p gow-core`)
- **Issue:** cargo 1.95 emits `error: invalid instruction `cargo:rustc-link-arg-bins` from build script of `gow-core` … The package gow-core v0.1.0 does not have a bin target`. This is exactly the situation Pitfall 4 describes: embed-manifest's directive targets bin outputs, and gow-core is lib-only in Phase 1.
- **Fix:** Added a `has_bin_target()` helper to build.rs that checks for `src/main.rs`, `src/bin/`, or a `[[bin]]` table in Cargo.toml. `embed_manifest::embed_manifest(...)` is only invoked when both `CARGO_CFG_WINDOWS` is set AND `has_bin_target()` returns true. The literal call to `ActiveCodePage::Utf8` + `Setting::Enabled` is retained inside the gated branch, so Phase 2+ utility bin crates can copy-paste build.rs verbatim.
- **Files modified:** `crates/gow-core/build.rs`
- **Verification:** `cargo build --workspace` finishes with zero errors and zero warnings; `cargo test -p gow-core` runs the smoke test and reports `1 passed`.
- **Committed in:** `c15706b`.

**4. [Rule 2 - Missing Critical] Added `init_does_not_panic` smoke test**
- **Found during:** Task 2 (after first green build)
- **Issue:** Runtime note required `cargo test -p gow-core` to be "green (init test passes; stubs compile)". Without a test the suite reports "0 tests passed" which is technically green but does not verify `init()` is callable or that the module graph is sound.
- **Fix:** Added `#[cfg(test)] mod tests { #[test] fn init_does_not_panic() { super::init(); } }` in `crates/gow-core/src/lib.rs`.
- **Files modified:** `crates/gow-core/src/lib.rs`
- **Verification:** `cargo test -p gow-core` → `1 passed; 0 failed`.
- **Committed in:** `c15706b`.

**5. [Rule 2 - Missing Critical] Added `.gitignore`**
- **Found during:** Task 2 commit preparation
- **Issue:** Without a gitignore, `target/` would be staged on first `git add .` by downstream contributors, polluting commits.
- **Fix:** Created `.gitignore` excluding `/target`, `**/target/`, `*.rs.bk`, `*.pdb`, OS/editor artifacts.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short` stopped reporting `target/` as untracked.
- **Committed in:** `c15706b`.

---

**Total deviations:** 5 auto-fixed (2 Rule 1 bug, 1 Rule 3 blocking, 2 Rule 2 missing critical)
**Impact on plan:** All five were required for the plan's own success criteria (`cargo build --workspace` passes, `cargo test -p gow-core` green). No scope creep: Plans 02/03 still own the full module implementations; Plan 04 still owns gow-probe and end-to-end manifest validation.

## Issues Encountered

- The plan's reference text and the research document both quoted API names (`LongPathAware::Yes`, `features = ["cargo"]` on assert_cmd) that do not exist in the pinned crate versions. Fixed inline via Rule 1 and documented above.
- Static-CRT + MSVC builds on this machine work with the pinned dependencies; no environment gap detected.

## Known Stubs

The plan explicitly requires stubs for six modules whose full implementations land in Plans 01-02 and 01-03. None of them are user-facing in this plan (no binaries exist yet), but they are documented here so the verifier does not flag them as oversights:

| File | Stub | Resolved By |
|------|------|-------------|
| `crates/gow-core/src/encoding.rs` | `setup_console_utf8` is a no-op placeholder on both Windows and non-Windows | Plan 01-02 (encoding/args/color) |
| `crates/gow-core/src/args.rs` | Module is empty apart from coverage comment | Plan 01-02 |
| `crates/gow-core/src/color.rs` | `enable_vt_mode` is a no-op placeholder | Plan 01-02 |
| `crates/gow-core/src/error.rs` | Module is empty apart from coverage comment | Plan 01-03 |
| `crates/gow-core/src/path.rs` | Module is empty apart from coverage comment | Plan 01-03 |
| `crates/gow-core/src/fs.rs` | Module is empty apart from coverage comment | Plan 01-03 |

The build.rs `embed_manifest()` call is similarly a design-time stub for gow-core (never executed because gow-core is lib-only) but a fully functional template for Phase 2+ utility bin crates.

## User Setup Required

None — no external service configuration required.

## Next Plan Readiness

- **01-02 (encoding, args, color):** Unblocked. Module files exist at the expected paths; `gow_core::init()` already wires the two functions that Plan 01-02 must implement (`encoding::setup_console_utf8`, `color::enable_vt_mode`).
- **01-03 (error, path, fs):** Unblocked. Modules exist; thiserror/path-slash/windows-sys are already in scope via workspace inheritance.
- **01-04 (gow-probe + integration tests):** Unblocked. build.rs is ready to embed the manifest as soon as Plan 04 adds a bin target to gow-core (or spawns a new gow-probe crate).
- No blockers or concerns for downstream Phase 2+ work.

## Self-Check

- [x] `D:\workspace\gow-rust\Cargo.toml` — exists, contains `resolver = "3"`.
- [x] `D:\workspace\gow-rust\.cargo\config.toml` — exists, contains `crt-static`.
- [x] `D:\workspace\gow-rust\.gitignore` — exists.
- [x] `D:\workspace\gow-rust\Cargo.lock` — exists.
- [x] `D:\workspace\gow-rust\crates\gow-core\Cargo.toml` — exists, contains `embed-manifest = "1.5"`.
- [x] `D:\workspace\gow-rust\crates\gow-core\build.rs` — exists, contains `ActiveCodePage::Utf8` and `long_path_aware`.
- [x] `D:\workspace\gow-rust\crates\gow-core\src\lib.rs` — exists, contains `pub fn init()` and all six `pub mod` declarations.
- [x] `D:\workspace\gow-rust\crates\gow-core\src\{args,color,encoding,error,fs,path}.rs` — all six stubs exist.
- [x] Commit `974a7fe` exists in `git log` (Task 1).
- [x] Commit `c15706b` exists in `git log` (Task 2).
- [x] `cargo build --workspace` → zero errors, zero warnings.
- [x] `cargo test -p gow-core` → 1 passed, 0 failed.
- [x] `cargo clippy -p gow-core -- -D warnings` → clean.

## Self-Check: PASSED

---
*Phase: 01-foundation*
*Plan: 01*
*Completed: 2026-04-20*
