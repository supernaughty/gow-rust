---
phase: 01-foundation
plan: 04
subsystem: gow-probe/integration
tags: [rust, assert_cmd, predicates, embed-manifest, integration-tests, gnu-compat, msys, gow-244, phase-1-capstone]

# Dependency graph
requires:
  - plan: 01-01
    provides: Cargo workspace, .cargo/config.toml (x86_64-pc-windows-msvc target triple), embed-manifest build-dep pattern, workspace-pinned assert_cmd/predicates/tempfile
  - plan: 01-02
    provides: gow_core::init(), gow_core::args::parse_gnu() (GNU exit-code 1 override), gow_core::encoding::setup_console_utf8(), gow_core::color::enable_vt_mode()
  - plan: 01-03
    provides: gow_core::path::try_convert_msys_path() (GOW #244 guard), gow_core::error::GowError, gow_core::fs::LinkType
provides:
  - gow-probe internal test binary (publish = false) exercising gow-core end-to-end via real spawned-process behavior
  - 9-test assert_cmd integration suite covering WIN-01, WIN-03, FOUND-02, FOUND-03, FOUND-06, and the GOW #244 regression at the binary level
  - Canonical embed-manifest build.rs template for Phase 2+ utility bin crates (unconditional on Windows, no bin-target gate needed because gow-probe has one)
  - Proof that `cargo test --workspace` is end-to-end green (46 tests: 34 gow-core unit + 9 gow-probe integration + 3 doctests, 0 pre-existing tests in gow-probe)
affects: [02-stateless, 03-filesystem, 04-text-processing, 05-search-navigation, 06-archive-network]

# Tech tracking
tech-stack:
  added: []  # All deps already pinned by Plan 01; Plan 04 only activated assert_cmd + predicates as dev-deps of a new crate
  patterns:
    - "gow-probe pattern: thin test binary that calls `gow_core::init()` first, then routes via clap subcommands to exercise individual gow-core primitives"
    - "Unconditional embed-manifest in bin crate build.rs — no bin-target gate needed (contrast: gow-core's lib-only build.rs must gate the call; see Plan 01)"
    - "assert_cmd integration pattern: `Command::cargo_bin(\"gow-probe\")` spawns the real debug binary under the active target triple; predicates::str::contains + .code(N) express GNU-shaped behavior assertions"
    - "Exit-code negative assertion pattern: `.stdout(predicate::str::contains(r\"C:\\\\\").not())` encodes the GOW #244 guard — bare `/c` must NOT appear as a converted drive letter"

key-files:
  created:
    - crates/gow-probe/Cargo.toml
    - crates/gow-probe/build.rs
    - crates/gow-probe/src/main.rs
    - crates/gow-probe/tests/integration.rs
  modified:
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "gow-probe is a non-shipped test-only crate. `publish = false` in Cargo.toml ensures it cannot be accidentally published to crates.io. It exists solely to give Phase 1 a runnable end-to-end verification artifact — Phase 2+ utility binaries will supersede its role once they exist."
  - "Subcommand dispatch in main.rs (`path <input>`, `exit-code <n>`, bare default) keeps the probe's CLI surface flat and each integration test can target one primitive without entangling them. The subcommand grammar mirrors what a real utility's clap definition would look like, so the `parse_gnu` doc/usage is also exercised."
  - "Integration tests cover WIN-03 (PowerShell compatibility) implicitly: assert_cmd spawns the binary via the same Win32 CreateProcessW that PowerShell uses. No dedicated pwsh-harness test is needed — the spawn-and-stdout loop is identical."
  - "The bare-`/c` GOW #244 guard is encoded as a negative assertion (`.not()`) rather than a positive equality check. This is intentional: the test asserts the bug cannot regress, not the exact stdout shape — future plans may decorate the output without breaking this guard."
  - "Target-triple binary location: `.cargo/config.toml` pins `x86_64-pc-windows-msvc`, so the binary lives at `target/x86_64-pc-windows-msvc/debug/gow-probe.exe` (not `target/debug/...`). assert_cmd's `cargo_bin` helper resolves this automatically via cargo metadata; the plan's acceptance criterion wording `target/debug/gow-probe.exe` is satisfied in spirit — the canonical `target/*/debug/` location is produced."

patterns-established:
  - "Pattern: phase capstone as a runnable binary — every foundation phase finishes with an executable that exercises the library end-to-end, not just unit tests, so the `cargo test --workspace` gate covers real process behavior"
  - "Pattern: embed-manifest build.rs in a bin crate is 1:1 identical to the gow-core template minus the `has_bin_target()` gate — Phase 2+ utility crates can copy-paste this build.rs verbatim"
  - "Pattern: GNU regression guards live alongside the positive behavior tests — the `/c` guard sits next to `/c/Users/foo` conversion so future readers see both cases together"

requirements-completed: [FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, FOUND-07, WIN-01, WIN-02, WIN-03]

# Metrics
duration: ~35min (Tasks 1+2 automated, then human verification gate)
completed: 2026-04-20
---

# Phase 1 Plan 04: gow-probe Integration Binary Summary

**Phase 1 capstone: `gow-probe` test binary and 9-test assert_cmd integration suite exercise every gow-core primitive end-to-end. `cargo test --workspace` reports 46 passed / 0 failed, closing out the foundation phase with runnable proof of WIN-01/WIN-02/WIN-03 and the GOW #244 path-conversion guard.**

## Performance

- **Duration:** ~35 min including human-verify checkpoint wait time (tasks 1–2 automated)
- **Tasks:** 2 automated + 1 human-verify checkpoint
- **Files created:** 4 (gow-probe crate — Cargo.toml, build.rs, src/main.rs, tests/integration.rs)
- **Files modified:** 2 (workspace Cargo.toml members list, Cargo.lock)
- **Tests added:** 9 integration tests (0 pre-existing in gow-probe)

## Accomplishments

- `gow-probe` test binary compiles and runs: `gow_core::init()` is the first line of `main()`, followed by clap-based subcommand dispatch for `path <input>`, `exit-code <n>`, and a bare-default init smoke test.
- 9 assert_cmd integration tests cover the Phase 1 observable surface: init smoke, explicit exit code 0/1, bad-flag → exit 1 (not clap's default 2), MSYS `/c` and `/d` drive conversion, relative-path passthrough, forward-slash normalization, and the GOW #244 bare-`/c` regression guard.
- `embed-manifest` build.rs is unconditional on Windows (no bin-target gate) — Phase 2+ utility crates can copy this build.rs verbatim.
- `publish = false` in `crates/gow-probe/Cargo.toml` prevents accidental crates.io publication — gow-probe is test-only infrastructure.
- Workspace test gate is fully green: **34 gow-core unit + 0 gow-probe unit + 9 gow-probe integration + 3 doctests = 46 tests pass**, 0 failures.
- Human verification checkpoint **approved** — all four manual PowerShell checks passed:
  1. `cargo run -p gow-probe` → `gow-probe: init ok`
  2. `cargo run -p gow-probe -- --unknown-flag; $LASTEXITCODE` → `1` (not clap's default 2)
  3. `cargo run -p gow-probe -- path /c/Users/foo` → `C:\Users\foo`
  4. `cargo run -p gow-probe -- path /c` → `/c` (GOW #244 guard, run in PowerShell to bypass MSYS shell expansion)

## Task Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | `42f2c33` | `feat(01-04): create gow-probe crate and add to workspace` — Cargo.toml members update + crates/gow-probe/{Cargo.toml, build.rs, src/main.rs} |
| 2 | `64d00f1` | `test(01-04): add gow-probe integration tests` — crates/gow-probe/tests/integration.rs with 9 assert_cmd tests |

_Plan metadata commit (this SUMMARY.md) lands separately as `docs(01-04): complete gow-probe integration binary plan`. STATE.md, ROADMAP.md, and REQUIREMENTS.md are intentionally **not** updated here — the orchestrator owns those after all Wave 3 work lands._

## Files Created

- `crates/gow-probe/Cargo.toml` — bin crate manifest; inherits workspace version/edition/rust-version/license/authors; `publish = false`; deps: gow-core (path), clap, anyhow; build-dep: embed-manifest 1.5; dev-deps: assert_cmd, predicates, tempfile (all workspace-pinned).
- `crates/gow-probe/build.rs` — embeds the Windows app manifest (activeCodePage=UTF-8, longPathAware=Enabled) unconditionally on Windows via `embed_manifest::embed_manifest(...)`. No bin-target gate needed — gow-probe is a bin crate.
- `crates/gow-probe/src/main.rs` — thin binary that calls `gow_core::init()` first, then dispatches via clap to `path <input>` (MSYS conversion), `exit-code <n>` (explicit exit), or the bare default (prints `gow-probe: init ok`).
- `crates/gow-probe/tests/integration.rs` — 9 `#[test]` functions using `assert_cmd::Command::cargo_bin("gow-probe")` + `predicates::prelude::*` to assert stdout content, exit codes, and (crucially) negative assertions for the GOW #244 regression guard.

## Files Modified

- `Cargo.toml` (workspace root) — added `"crates/gow-probe"` to the `[workspace] members = [...]` list.
- `Cargo.lock` — regenerated to include the new gow-probe crate in the dependency graph (no new external deps pulled; clap/anyhow/assert_cmd/predicates/tempfile/embed-manifest were all already in the lockfile from Plans 01–03).

## Test Results

### Workspace-level

```text
cargo test --workspace

running 34 tests                          # gow-core unit tests
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests                           # gow-probe unit tests (none; integration-only)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 9 tests                           # gow-probe integration tests
test test_default_init_ok ................................. ok
test test_explicit_exit_code_zero .......................... ok
test test_bad_flag_exits_1_not_2 ........................... ok
test test_path_bare_drive_not_converted .................... ok   (GOW #244 guard)
test test_explicit_exit_code_one ........................... ok
test test_path_msys_c_drive_conversion ..................... ok
test test_path_msys_d_drive_conversion ..................... ok
test test_path_windows_forward_slash_normalized ............ ok
test test_path_relative_unchanged .......................... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Doc-tests gow_core                        # args::parse_gnu + error::io_err + fs::normalize_junction_target
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Total: 46 tests passed, 0 failed.**

### Human verification checkpoint (all PASSED — user verdict: "Approved")

| # | Command | Expected | Observed |
|---|---------|----------|----------|
| 1 | `cargo run -p gow-probe` | `gow-probe: init ok` | `gow-probe: init ok` |
| 2 | `cargo run -p gow-probe -- --unknown-flag; $LASTEXITCODE` | `1` | `1` |
| 3 | `cargo run -p gow-probe -- path /c/Users/foo` | `C:\Users\foo` | `C:\Users\foo` |
| 4 | `cargo run -p gow-probe -- path /c` (PowerShell) | `/c` | `/c` |

### Clippy

```text
cargo clippy -p gow-probe -- -D warnings
  Checking anyhow v1.0.102
  Finished `dev` profile — zero warnings
```

## Decisions Made

- **`publish = false` is mandatory.** gow-probe is test infrastructure, not a user-facing utility. Committing `publish = false` at the top of Cargo.toml is the defense-in-depth guard that makes accidental crates.io publication impossible even if someone runs `cargo publish -p gow-probe` by mistake. T-04-01 in the plan's threat register is mitigated by exactly this line.
- **Integration tests live in `tests/`, not `src/`.** Rust's convention: each `.rs` file under `crates/gow-probe/tests/` is compiled as a separate test binary and sees gow-probe as an external black-box crate via `assert_cmd::Command::cargo_bin(...)`. This is stronger than unit tests because it exercises the actual compiled binary's process behavior (argv parsing, exit codes, stdout/stderr), not just library calls.
- **PowerShell compatibility (WIN-03) is covered implicitly.** assert_cmd uses Win32 `CreateProcessW` under the hood — the same syscall PowerShell uses to spawn child processes. If the 9 integration tests pass, the binary is guaranteed to behave identically when spawned from pwsh. No dedicated `pwsh.exe -Command "..."` harness is needed.
- **Bare `/c` guard uses a negative predicate.** `test_path_bare_drive_not_converted` asserts `.stdout(predicate::str::contains(r"C:\").not())`. This is the correct assertion shape for a regression guard: the test says "the bug cannot come back" rather than "the output exactly equals this string", which leaves room for future output decoration (color codes, trailing newline variants) without requiring a test rewrite.

## Deviations from Plan

None during automated tasks. Tasks 1 and 2 executed exactly as specified:

- Files, function signatures, and test assertions match the plan text verbatim (including the `path <input>` / `exit-code <n>` subcommand grammar and the 9-test integration matrix).
- The workspace-root Cargo.toml members list update landed in the same commit as the gow-probe Cargo.toml creation (Task 1, `42f2c33`), matching the plan's single-commit scope for that task.
- No Rule 1/2/3 auto-fixes were needed — the code compiled green on first build, and all 9 tests passed on first run.
- The plan's acceptance criterion mentions `target/debug/gow-probe.exe`. The actual binary lives at `target/x86_64-pc-windows-msvc/debug/gow-probe.exe` because `.cargo/config.toml` pins an explicit target triple (Plan 01 decision). This is the canonical location; `assert_cmd::Command::cargo_bin(...)` resolves it via cargo metadata, so no test changes were needed. Not flagged as a deviation — the spirit of the criterion (binary produced in a `target/*/debug/` location) is satisfied.

## Issues Encountered

None. Both automated tasks passed verification on the first run. The human-verify checkpoint was approved without any re-work.

## Auth Gates

None — all work is local. No external services, no credentials, no tokens.

## Known Stubs

None. Phase 1 is complete: all six gow-core modules have full implementations (Plans 02, 03) and gow-probe verifies the end-to-end behavior. No stubs remain in gow-core or gow-probe.

## Threat Flags

No new security surface introduced beyond the plan's own threat register. Specifically:

- T-04-01 (Tampering — gow-probe publication risk): **mitigated** by `publish = false` in `crates/gow-probe/Cargo.toml` line 9.
- T-04-02 (Tampering — GOW #244 path conversion): **mitigated** by `test_path_bare_drive_not_converted` with the `.not()` negative assertion in `crates/gow-probe/tests/integration.rs`.
- T-04-03 (Denial of Service — build.rs manifest embedding): **accepted** — build failure is a loud compile error via `.expect("unable to embed manifest")`, not a silent bad binary.

No new endpoints, no auth paths, no schema changes, no new trust boundaries.

## User Setup Required

None. `cargo test --workspace` runs on a clean clone with the existing toolchain (rustc 1.95.0 MSVC) — no extra install, no privilege elevation, no environment variables.

## Next Plan Readiness

- **Phase 2 (stateless utilities — head/tail/wc/cat/echo/seq):** Unblocked. All gow-core primitives (args::parse_gnu, encoding::setup_console_utf8, color::{enable_vt_mode, color_choice, stdout}, error::{GowError, io_err}, path::{try_convert_msys_path, normalize_file_args, to_windows_path}, fs::{LinkType, link_type, normalize_junction_target}) are stable public API and exercised end-to-end. Phase 2 utility crates can copy `crates/gow-probe/build.rs` verbatim as their manifest-embedding template.
- **MSI installer (future phase):** The `.cargo/config.toml` + `+crt-static` + embed-manifest pattern produces self-contained binaries with no VCRUNTIME dependency and correct longPathAware/activeCodePage metadata — MSI packaging has exactly what it needs.
- **CI pipeline (if/when added):** `cargo test --workspace` is a single green gate that covers 46 test cases across the full foundation. A GitHub Actions `windows-latest` job running this one command fully validates Phase 1.

## Self-Check

- [x] `D:\workspace\gow-rust\crates\gow-probe\Cargo.toml` — exists (596 bytes), contains `publish = false` and `embed-manifest = "1.5"` under `[build-dependencies]`.
- [x] `D:\workspace\gow-rust\crates\gow-probe\build.rs` — exists (1092 bytes), contains `ActiveCodePage::Utf8` and `Setting::Enabled` (long_path_aware).
- [x] `D:\workspace\gow-rust\crates\gow-probe\src\main.rs` — exists (2038 bytes), contains `gow_core::init()`, `gow_core::args::parse_gnu(`, and subcommand dispatch for `path` / `exit-code`.
- [x] `D:\workspace\gow-rust\crates\gow-probe\tests\integration.rs` — exists (4042 bytes), contains `assert_cmd::Command`, `cargo_bin("gow-probe")`, `test_bad_flag_exits_1_not_2` with `.code(1)`, `test_path_msys_c_drive_conversion`, and `test_path_bare_drive_not_converted` with `.not()`.
- [x] Commit `42f2c33` exists in `git log` (Task 1 — crate + workspace wire-up).
- [x] Commit `64d00f1` exists in `git log` (Task 2 — integration tests).
- [x] `cargo build --workspace` → zero errors, zero warnings (finished in 2.72s).
- [x] `cargo test --workspace` → 34 + 0 + 9 + 3 = **46 passed, 0 failed, 0 ignored**.
- [x] `cargo clippy -p gow-probe -- -D warnings` → clean, zero warnings.
- [x] Human-verify checkpoint: all 4 manual PowerShell checks passed; user verdict "Approved — all four checks passed".

## Self-Check: PASSED

---
*Phase: 01-foundation*
*Plan: 04*
*Completed: 2026-04-20*
