---
phase: "02"
plan: "04"
---

# T04: Plan 04

**# Phase 2 Plan 04: gow-pwd — GNU pwd with -L/-P and UNC-safe canonicalization**

## What Happened

# Phase 2 Plan 04: gow-pwd — GNU pwd with -L/-P and UNC-safe canonicalization

**One-liner:** Ships `pwd.exe` with GNU -L/-P flag parity, PWD env fallback with canonicalize-based validation, and inline UNC-safe `\\?\` prefix stripping (dunce::simplified rule, no crate dep).

## Objective

Phase 2 UTIL-02. GNU `pwd` is trivial on Unix but non-trivial on Windows because `std::fs::canonicalize` always returns the extended-length `\\?\` prefix — and the naive strip corrupts true UNC paths (`\\?\UNC\server\share`). This plan:

1. Implements the dunce::simplified safety rule inline as `simplify_canonical` with 9 unit tests including the critical `preserves_unc_prefix` regression guard (T-02-04-02 mitigation).
2. Wires a full `uumain` with -L/-P flags, `$PWD` preference with canonicalize-based validation (T-02-04-01 mitigation), and GNU-format error reporting.
3. Ships 8 integration tests proving `-P` never emits `\\?\` on drive-letter paths, `-L` === default, and bad flags exit with code 1 in GNU `pwd: …` format.

Delivers: `pwd.exe` observable via `pwd -P` printing a Windows drive-letter path.

## What Was Built

### Task 1 — `src/canonical.rs` (commit `b3b2415`)

File: `crates/gow-pwd/src/canonical.rs` (109 lines). Single public function `simplify_canonical(p: &Path) -> PathBuf` implementing the dunce-equivalent rule:

> Strip `\\?\` only if the 5th–7th bytes are `[A-Za-z]:\`. Preserve everything else, including `\\?\UNC\…`, `\\?\GLOBALROOT\…`, paths without the prefix, and malformed prefix-only input.

The implementation is a byte-slice check — no regex, no allocations beyond the final `PathBuf`. Non-UTF-8 paths (unusual on modern Windows) are returned verbatim.

Nine unit tests in the same file:

| Test | Input | Expected |
|------|-------|----------|
| `strips_drive_letter_prefix_uppercase` | `\\?\C:\Users\foo` | `C:\Users\foo` |
| `strips_drive_letter_prefix_lowercase` | `\\?\c:\Users\foo` | `c:\Users\foo` |
| `strips_minimal_drive_letter_path` | `\\?\D:\` | `D:\` |
| `preserves_unc_prefix` **(T-02-04-02)** | `\\?\UNC\server\share\file` | verbatim |
| `preserves_nt_device_path` | `\\?\GLOBALROOT\Device\Foo` | verbatim |
| `passthrough_no_prefix` | `C:\Users\foo` | verbatim |
| `passthrough_relative_path` | `relative\path` | verbatim |
| `preserves_too_short_prefix_only` | `\\?\` | verbatim (malformed fallback) |
| `preserves_prefix_without_drive_letter` | `\\?\1:\file` | verbatim (non-alpha drive) |

Task-1 `simplify_canonical` was annotated `#[allow(dead_code)]` so clippy stayed clean while lib.rs was still the Plan-01 stub; Task 2 removed that annotation.

### Task 2 — lib.rs + build.rs + Cargo.toml + integration tests (commit `c7c5ea1`)

**`crates/gow-pwd/build.rs`** (verbatim copy of `crates/gow-probe/build.rs`, doc string updated to name gow-pwd): emits `cargo:rerun-if-changed` markers and calls `embed_manifest::embed_manifest(new_manifest("Gow.Rust").active_code_page(Utf8).long_path_aware(Enabled))` on Windows. No `has_bin_target()` gate (D-16c).

**`crates/gow-pwd/Cargo.toml`**: added `[build-dependencies] embed-manifest = "1.5"` and `[dev-dependencies]` block with assert_cmd, predicates, tempfile (all `{ workspace = true }`).

**`crates/gow-pwd/src/lib.rs`** — full uumain. Dispatch:

```rust
let cwd = if physical {
    match std::env::current_dir().and_then(std::fs::canonicalize) {
        Ok(p) => simplify_canonical(&p),
        Err(e) => { eprintln!("pwd: {e}"); return 1; }
    }
} else {
    std::env::var_os("PWD")
        .map(PathBuf::from)
        .filter(|p| validate_pwd(p))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
};
println!("{}", cwd.display());
```

`validate_pwd` canonicalizes both PWD and current_dir and compares; any error or mismatch returns false so the caller falls through to `current_dir()`. This is the T-02-04-01 mitigation — a hostile `PWD=/etc/shadow` cannot masquerade as the real cwd.

**`crates/gow-pwd/tests/integration.rs`** (8 tests, all `assert_cmd + predicates`):

1. `test_default_prints_cwd` — default mode contains the tempdir basename in stdout.
2. `test_p_flag_strips_unc_prefix` — `-P` stdout never starts with `\\?\`.
3. `test_p_flag_returns_drive_letter_form` — `-P` stdout matches regex `^[A-Z]:\\`.
4. `test_l_flag_same_as_default` — `-L` and default produce byte-identical stdout.
5. `test_bad_flag_exits_1_not_2` — `--completely-unknown-xyz` exits 1 (not clap's 2).
6. `test_gnu_error_format_on_bad_flag` — stderr starts with `pwd:`.
7. `test_output_has_trailing_newline` — every successful call ends with `\n`.
8. `test_pwd_env_used_when_valid` — explicit PWD=<tempdir> yields a line containing the tempdir basename.

## Verification Evidence

```
$ cargo build -p gow-pwd
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.04s   (0 warnings)

$ cargo test -p gow-pwd
running 9 tests
test canonical::tests::passthrough_no_prefix ... ok
test canonical::tests::preserves_unc_prefix ... ok
test canonical::tests::preserves_prefix_without_drive_letter ... ok
test canonical::tests::strips_drive_letter_prefix_lowercase ... ok
test canonical::tests::passthrough_relative_path ... ok
test canonical::tests::preserves_nt_device_path ... ok
test canonical::tests::strips_drive_letter_prefix_uppercase ... ok
test canonical::tests::strips_minimal_drive_letter_path ... ok
test canonical::tests::preserves_too_short_prefix_only ... ok
test result: ok. 9 passed; 0 failed

running 8 tests
test test_gnu_error_format_on_bad_flag ... ok
test test_bad_flag_exits_1_not_2 ... ok
test test_output_has_trailing_newline ... ok
test test_default_prints_cwd ... ok
test test_p_flag_returns_drive_letter_form ... ok
test test_p_flag_strips_unc_prefix ... ok
test test_pwd_env_used_when_valid ... ok
test test_l_flag_same_as_default ... ok
test result: ok. 8 passed; 0 failed

$ cargo clippy -p gow-pwd -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s   (clean)
```

### Sample output (from `C:\Users`)

```
$ cd C:/Users && target/.../pwd.exe
C:/Users            # default = $PWD from bash (forward slashes, verbatim)

$ cd C:/Users && target/.../pwd.exe -P
C:\Users            # -P = canonicalized + \\?\ stripped (backslashes, native form)

$ pwd.exe --bad
pwd: error: unexpected argument '--bad' found
                    # exit code 1 (GNU `pwd: ...` format)
```

Note: when bash sets `PWD=C:/Users` and validate_pwd canonicalizes both sides (both resolve to `\\?\C:\Users`), PWD wins and is printed verbatim — which is why default shows forward slashes while `-P` shows backslashes. This is the correct GNU `pwd` semantic.

## Acceptance Criteria — Task-by-Task

### Task 1

- [x] `cargo test -p gow-pwd --lib canonical` → 9 tests pass
- [x] `crates/gow-pwd/src/canonical.rs` contains `pub fn simplify_canonical`
- [x] `preserves_unc_prefix` passes — UNC safety guaranteed
- [x] `cargo clippy -p gow-pwd --lib -- -D warnings` exits 0

### Task 2

- [x] `cargo test -p gow-pwd` → 17 tests total (9 unit + 8 integration), all pass
- [x] `cargo run -p gow-pwd -- -P` prints path NOT starting with `\\?\`
- [x] `cargo run -p gow-pwd -- -P` prints path matching `^[A-Za-z]:\\`
- [x] `cargo run -p gow-pwd -- --bad` exits with code 1
- [x] `cargo clippy -p gow-pwd -- -D warnings` exits 0
- [x] `src/lib.rs` contains `mod canonical;` and calls `simplify_canonical(&p)`
- [x] `src/lib.rs` calls `gow_core::init()` as first line of `uumain`

### Plan-level

- [x] 17 tests pass (9 unit + 8 integration)
- [x] `pwd -P` output never starts with `\\?\` on drive-letter paths
- [x] `pwd` default prefers `$PWD` when canonicalize-equal to current_dir
- [x] UNC network paths (`\\?\UNC\...`) preserved (verified by unit test `preserves_unc_prefix`)
- [x] Windows manifest embedded (build.rs); clippy clean
- [x] No modifications outside `crates/gow-pwd/` (plus auto-updated `Cargo.lock`)

## Threat Model Status

| Threat ID | Mitigation |
|-----------|------------|
| T-02-04-01 (Tampering / malicious `$PWD`) | ✅ `validate_pwd` canonicalizes both sides + equality check; any failure rejects PWD. |
| T-02-04-02 (Information Disclosure / UNC strip corruption) | ✅ `simplify_canonical` preserves `\\?\UNC\…` verbatim; `preserves_unc_prefix` unit test guards against regression. |
| T-02-04-03 (Information Disclosure / canonicalize follows symlink) | ✅ Accepted by design — GNU `pwd -P` is defined to follow symlinks. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Added `mod canonical;` to lib.rs during Task 1**

- **Found during:** Task 1 verification (`cargo test -p gow-pwd --lib canonical`).
- **Issue:** Task 1 creates `src/canonical.rs` with unit tests, but Cargo doesn't compile the module unless declared in lib.rs. Without `mod canonical;`, the 9 unit tests would not run, so Task 1's `cargo test -p gow-pwd --lib canonical` acceptance check would pass vacuously (0 tests executed) — a silent regression risk.
- **Fix:** Added `mod canonical;` to the Plan-01 stub lib.rs in Task 1's commit. Task 2 rewrote lib.rs wholesale, so the temporary bridge was self-contained.
- **Files modified:** `crates/gow-pwd/src/lib.rs` (in Task 1 commit `b3b2415`).

**2. [Rule 3 — Blocking] `#[allow(dead_code)]` on `simplify_canonical` during Task 1**

- **Found during:** Task 1 clippy check — `error: function simplify_canonical is never used`.
- **Issue:** Task 1 lands the module with tests but the lib.rs stub doesn't consume it yet (that comes in Task 2). `clippy -D warnings` rejects dead public functions.
- **Fix:** Added `#[allow(dead_code)] // wired by lib.rs uumain in Task 2` to the function definition. Task 2 removed the annotation in the same commit that added the `use crate::canonical::simplify_canonical;` import, keeping the source tree lint-clean at every commit boundary.
- **Files modified:** `crates/gow-pwd/src/canonical.rs` (Task 1 commit added the annotation; Task 2 commit removed it).

No Rule 1 bugs, no Rule 2 missing functionality beyond what the plan specified, no Rule 4 architectural changes. `validate_pwd` was kept at full fidelity (not simplified).

## Authentication Gates

None — purely local filesystem + cargo.

## Commits

| Hash | Type | Summary |
|------|------|---------|
| `b3b2415` | feat | Add simplify_canonical with UNC-safe \\?\ prefix strip (Task 1) |
| `c7c5ea1` | feat | Implement pwd with -L/-P flags and PWD env fallback (Task 2) |

## Self-Check: PASSED

**Files verified on disk:**

- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\crates\gow-pwd\build.rs`
- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\crates\gow-pwd\Cargo.toml` (embed-manifest build-dep added)
- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\crates\gow-pwd\src\canonical.rs` (pub fn simplify_canonical)
- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\crates\gow-pwd\src\lib.rs` (mod canonical; + uumain + validate_pwd)
- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\crates\gow-pwd\tests\integration.rs` (8 assert_cmd tests)
- FOUND: `D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81\.planning\phases\02-stateless\02-04-SUMMARY.md` (this file)

**Commits verified in git log:**

- FOUND: `b3b2415` feat(02-04): add simplify_canonical with UNC-safe \\?\ prefix strip
- FOUND: `c7c5ea1` feat(02-04): implement pwd with -L/-P flags and PWD env fallback

**Build/test gates verified:**

- `cargo build -p gow-pwd` → exit 0, 0 warnings
- `cargo test -p gow-pwd` → 9 + 8 = 17 tests pass, 0 failed
- `cargo clippy -p gow-pwd -- -D warnings` → exit 0
- `cargo run -p gow-pwd -- -P` prints backslash drive-letter path (`D:\workspace\gow-rust\.claude\worktrees\agent-a14aff81`)
- `cargo run -p gow-pwd -- --bad` exits 1 with `pwd: error: …`
- `git diff --name-only BASE..HEAD` shows only `crates/gow-pwd/` paths + `Cargo.lock` — no out-of-scope edits

All plan-level success criteria satisfied.
