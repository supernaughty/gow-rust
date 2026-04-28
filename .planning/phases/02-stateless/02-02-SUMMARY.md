---
phase: "02"
plan: "02"
---

# T02: Plan 02

**# Phase 2 Plan 02: true + false + yes Summary**

## What Happened

# Phase 2 Plan 02: true + false + yes Summary

**gow-true / gow-false ship as 3-line D-22 exit-code returners with embedded Windows manifests; gow-yes ships with a 16 KiB prefill-buffer + write_all-loop implementation that silently handles BrokenPipe and round-trips UTF-8 argv — all three verified by 20 passing tests and a `yes | head -c 1000` exactness check.**

## Performance

- **Duration:** ~7 min (Task 1 + Task 2 RED + Task 2 GREEN, including cargo cold-compile cycles)
- **Completed:** 2026-04-21
- **Tasks:** 2 (Task 2 executed as TDD RED + GREEN = 2 commits)
- **Files created:** 7 (3 build.rs + 3 tests/integration.rs + this SUMMARY.md)
- **Files modified:** 4 (3 Cargo.toml + 1 src/lib.rs, plus Cargo.lock)

## Accomplishments

- **UTIL-08 (true)** and **UTIL-09 (false)** locked in at final D-22 behavior with 5 integration tests each covering no-args, junk-args, UTF-8 argv, silent stdout/stderr, and `--` separator.
- **UTIL-07 (yes)** implements the RESEARCH.md Q4 throughput pattern verbatim: 16 KiB heap buffer prefilled via `prepare_buffer`, locked stdout, `write_all` loop, and silent exit 0 on `ErrorKind::BrokenPipe`.
- All 3 crates now embed the Windows application manifest (UTF-8 active code page + long-path aware) via a per-crate `build.rs` copied verbatim from the `gow-probe` template.
- 20 tests total across the 3 crates — all green; `cargo clippy -p gow-true -p gow-false -p gow-yes --all-targets -- -D warnings` is clean.

## Task Commits

Each task was committed atomically with `--no-verify` (pre-commit hooks run post-wave by the orchestrator per the parallel-executor contract):

1. **Task 1: gow-true + gow-false final scaffold** — `1d63683` (feat)
   - Added build.rs to both crates (verbatim gow-probe copy)
   - Expanded Cargo.toml with [build-dependencies] embed-manifest + [dev-dependencies] assert_cmd + predicates
   - Added tests/integration.rs (5 tests each)
   - Kept the 3-line D-22 final impl in src/lib.rs untouched
2. **Task 2 RED: gow-yes failing integration tests** — `4c62b5b` (test)
   - Added build.rs (verbatim gow-probe copy)
   - Expanded Cargo.toml with [build-dependencies] + [dev-dependencies]
   - Added 6 integration tests covering default `y\n`, multi-arg join (2 + 3 args), single-arg, UTF-8 argv, BrokenPipe exit 0
   - All 6 tests failed with `UnexpectedEof` against the stub — RED confirmed
3. **Task 2 GREEN: gow-yes real uumain** — `b440565` (feat)
   - Replaced stub with uumain + `run` inner + `prepare_buffer` helper
   - Added 4 unit tests for `prepare_buffer` (short-input fill, default `y\n` fill, long-input passthrough, boundary at exactly half size)
   - All 10 tests green; `cargo clippy -D warnings` clean

**Plan metadata:** this SUMMARY will be bundled with STATE.md / ROADMAP.md updates by the orchestrator post-wave, not in this executor.

## Files Created/Modified

### Created

- `crates/gow-true/build.rs` — Windows manifest embedder (doc comment: "Build script for gow-true."). 25 lines.
- `crates/gow-true/tests/integration.rs` — 5 tests: no-args, junk-args, double-dash, UTF-8 arg, silent stdout/stderr. 45 lines.
- `crates/gow-false/build.rs` — Same as gow-true's build.rs with "gow-false" in doc comment. 25 lines.
- `crates/gow-false/tests/integration.rs` — 5 tests mirroring true but asserting `.failure()` + `.code(1)`. 45 lines.
- `crates/gow-yes/build.rs` — Same build.rs template with "gow-yes" in doc comment. 25 lines.
- `crates/gow-yes/tests/integration.rs` — 6 tests via a `capture_prefix(args, n_bytes)` helper that spawns yes via `std::process::Command`, reads a bounded prefix, then kills the child. 129 lines.

### Modified

- `crates/gow-true/Cargo.toml` — added [build-dependencies] embed-manifest = "1.5"; [dev-dependencies] assert_cmd + predicates (workspace = true).
- `crates/gow-false/Cargo.toml` — same as gow-true.
- `crates/gow-yes/Cargo.toml` — same additions; kept existing clap + anyhow + thiserror for future `--help` / `--version` support.
- `crates/gow-yes/src/lib.rs` — stub → real uumain. 134 lines including 4 unit tests.

## Verification Evidence

### Automated tests

```text
cargo test -p gow-true -p gow-false -p gow-yes

gow-true:
  tests/integration.rs: 5 passed
  (unit/main/doctest: 0 tests each — expected)

gow-false:
  tests/integration.rs: 5 passed
  (unit/main/doctest: 0 tests each — expected)

gow-yes:
  unittests src/lib.rs: 4 passed  (prepare_buffer)
  tests/integration.rs: 6 passed  (default y, 2-arg, 3-arg, 1-arg, UTF-8, BrokenPipe)
  (main/doctest: 0 tests each — expected)

Total: 20 passed, 0 failed.
```

```text
cargo clippy -p gow-true -p gow-false -p gow-yes --all-targets -- -D warnings
  Finished `dev` profile in 0.14s   (zero warnings, zero errors)
```

### Manual smoke

```text
./target/x86_64-pc-windows-msvc/debug/true.exe  ; echo $?
  → exit 0

./target/x86_64-pc-windows-msvc/debug/false.exe ; echo $?
  → exit 1

./target/x86_64-pc-windows-msvc/debug/yes.exe 2>/dev/null | head -c 6
  y
  y
  y
  (exactly `y\ny\ny\n`, 6 bytes)

./target/x86_64-pc-windows-msvc/debug/yes.exe hello world 2>/dev/null | head -c 26
  hello world
  hello world
  he
  (2 full lines + 2-byte prefix of the 3rd; 26 bytes total)

./target/x86_64-pc-windows-msvc/debug/yes.exe 안녕 2>/dev/null | head -c 14
  안녕
  안녕
  (2 full lines; 안녕 is 6 UTF-8 bytes + '\n' = 7 bytes per line)

./target/x86_64-pc-windows-msvc/debug/yes.exe 2>/dev/null | head -c 1000 | wc -c
  1000
  (BrokenPipe path: stdout captures exactly 1000 bytes before head closes the pipe)
```

### BrokenPipe exit-code verification

`tests/integration.rs::test_broken_pipe_exits_zero` spawns yes with `Stdio::piped()`, reads 64 bytes, drops the reader (closes the pipe), then `child.wait()`. The test asserts `status.code() == Some(0)`. Verified green. This is the authoritative check since shell-pipeline exit codes on Windows Git Bash can be muddled by PIPE-signal-aware bash translating SIGPIPE into exit 141 for its own bookkeeping — our test reads the child's exit directly.

## Decisions Made

- **Use `std::process::Command` + `Stdio::piped()` + `child.kill()` for infinite-loop tests, not `assert_cmd::Command::timeout()`.** The plan's action note warned about this: assert_cmd 2.x `.timeout()` calls `wait_timeout_process`; it does not actively kill the process. For an infinite-loop program like `yes`, `.output()` would block forever even with a timeout set. The direct-Command + bounded-read + kill pattern is deterministic and fast.
- **`BUF_SIZE = 16 * 1024`, heap-allocated via `Vec<u8>`.** RESEARCH.md Q4 notes 16 KiB hits ~2 GB/s (plenty for any downstream consumer); 64 KiB would require heap anyway (stack allocation unsafe at that size). 16 KiB is the documented sweet spot.
- **`prepare_buffer` is `pub(crate)`, not `pub`.** Internal helper; exposed only to the `#[cfg(test)]` module inside the same crate. Avoids committing to a semver-stable API for a helper that may move or change.
- **`gow-true` / `gow-false` intentionally skip `gow_core::init()`.** D-22 says the body is just `return 0` / `return 1`. Calling `init()` would set up UTF-8 console mode for binaries that emit zero bytes — wasted cycles on a hot path (true/false invocations dominate shell-script overhead).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced `std::iter::repeat(*b"y\\n").take(16)` with `std::iter::repeat_n(*b"y\\n", 16)`**

- **Found during:** Task 2 RED, running `cargo clippy --all-targets -- -D warnings` on the new test file.
- **Issue:** Clippy on the current workspace toolchain reports `repeat(...).take(n)` as a warning / rustc edition-2024 lint and fails under `-D warnings`. The plan's example snippet used the old-form pattern.
- **Fix:** Replaced with `std::iter::repeat_n(*b"y\\n", 16).flatten().collect::<Vec<u8>>()`. Identical semantics (finite repeat N times).
- **Files modified:** `crates/gow-yes/tests/integration.rs`
- **Verification:** Clippy green (`cargo clippy -p gow-yes --all-targets -- -D warnings` exits 0); test green.
- **Committed in:** `4c62b5b` (Task 2 RED commit).

**2. [Rule 3 - Blocking] Included `Cargo.lock` in each task commit**

- **Found during:** Task 1 post-edit `git status` check.
- **Issue:** Expanding `[dependencies]` and adding `[build-dependencies]` on the 3 crates caused cargo to update `Cargo.lock` (now listing embed-manifest / assert_cmd / predicates under those crates). Leaving `Cargo.lock` uncommitted would cause subsequent `cargo build` cycles to re-resolve and produce spurious diffs in other plans' commits.
- **Fix:** Added `Cargo.lock` to both Task 1 and Task 2 RED commits (only the minimal generated diff — no manual lock-file edits).
- **Files modified:** `Cargo.lock`
- **Verification:** `git status --short` clean after each commit; subsequent `cargo build` produces no lockfile diff.
- **Committed in:** `1d63683` (Task 1) and `4c62b5b` (Task 2 RED) — small incidental edits bundled with their trigger commit.

---

**Total deviations:** 2 auto-fixed (1 Rule 1 clippy-edition lint, 1 Rule 3 blocking lockfile sync)
**Impact on plan:** Both deviations were zero-scope additions required for a clean `cargo clippy -D warnings` gate and a clean worktree. No architectural or behavioral changes. Plan-level acceptance criteria met exactly as written.

## Issues Encountered

None beyond the two deviations above. The RED → GREEN transition was clean — all 6 integration tests failed with the expected `UnexpectedEof` against the stub (stub exits 1 immediately, so `read_exact(&mut [0u8; N])` can never fill its buffer), and all 6 + 4 unit tests passed on the first run after replacing the stub body.

## Plan Success Criteria — Checklist

- [x] `cargo build -p gow-true -p gow-false -p gow-yes` succeeds with zero warnings
- [x] `cargo test -p gow-true && cargo test -p gow-false && cargo test -p gow-yes` — 20 tests green (5 + 5 + 10)
- [x] `cargo clippy -p gow-true -p gow-false -p gow-yes -- -D warnings` clean
- [x] `yes | head -c 1000 | wc -c` produces exactly 1000 bytes (BrokenPipe exit 0 verified in test_broken_pipe_exits_zero)
- [x] gow-true returns 0, gow-false returns 1 on every invocation (verified via 5 tests each)
- [x] gow-yes default output is `y\n` repeated; multi-arg joins with spaces; BrokenPipe → exit 0
- [x] All three crates have embedded Windows manifest via build.rs
- [x] Integration test count meets ≥ 5 per crate (5 + 5 + 6 integration + 4 unit = 20 tests)
- [x] SUMMARY.md committed to `.planning/phases/02-stateless/02-02-SUMMARY.md` with Self-Check: PASSED
- [x] No modifications to STATE.md, ROADMAP.md, REQUIREMENTS.md, or anything outside `crates/gow-{true,false,yes}/` + workspace `Cargo.lock` + this SUMMARY

## TDD Gate Compliance

Plan 02-02 is not plan-level `type: tdd`, but Task 2 is `tdd="true"`. Gate sequence verified in git log:

- RED gate: `4c62b5b` — `test(02-02): add failing integration tests for gow-yes (RED)` — 6 tests failing with UnexpectedEof
- GREEN gate: `b440565` — `feat(02-02): implement gow-yes with 16 KiB prefill buffer (GREEN)` — 6 integration + 4 unit tests passing
- REFACTOR: none needed; implementation already matches RESEARCH.md Q4 reference and passes clippy -D warnings on first writing.

## Next Phase Readiness

Waves 2.3, 2.4, 2.5 (echo, pwd, basename + dirname) can proceed independently. They share:

- The same build.rs template (copy-verbatim with only the line-1 doc comment changed).
- The same Cargo.toml expansion pattern for [build-dependencies] / [dev-dependencies].
- The option to reuse the `capture_prefix`-style integration-test helper if they ever need bounded reads (likely not — only yes loops forever).

Wave 3 (mkdir, rmdir, touch) and Wave 4 (env, tee, wc, which) are unaffected — they depend on gow-core's init/args/path APIs, not on anything from this plan.

## Self-Check: PASSED

**Files verified on disk:**

- FOUND: `D:\workspace\gow-rust\crates\gow-true\build.rs` (25 lines)
- FOUND: `D:\workspace\gow-rust\crates\gow-true\tests\integration.rs` (45 lines, 5 tests)
- FOUND: `D:\workspace\gow-rust\crates\gow-true\Cargo.toml` (modified: [build-dependencies] + [dev-dependencies] present)
- FOUND: `D:\workspace\gow-rust\crates\gow-false\build.rs` (25 lines)
- FOUND: `D:\workspace\gow-rust\crates\gow-false\tests\integration.rs` (45 lines, 5 tests)
- FOUND: `D:\workspace\gow-rust\crates\gow-false\Cargo.toml` (modified: [build-dependencies] + [dev-dependencies] present)
- FOUND: `D:\workspace\gow-rust\crates\gow-yes\build.rs` (25 lines)
- FOUND: `D:\workspace\gow-rust\crates\gow-yes\tests\integration.rs` (129 lines, 6 tests)
- FOUND: `D:\workspace\gow-rust\crates\gow-yes\Cargo.toml` (modified: [build-dependencies] + [dev-dependencies] present)
- FOUND: `D:\workspace\gow-rust\crates\gow-yes\src\lib.rs` (134 lines, 4 unit tests, prepare_buffer + uumain + run)
- FOUND: `D:\workspace\gow-rust\.planning\phases\02-stateless\02-02-SUMMARY.md` (this file)

**Commits verified in git log:**

- FOUND: `1d63683` feat(02-02): finalize gow-true and gow-false with build.rs and integration tests
- FOUND: `4c62b5b` test(02-02): add failing integration tests for gow-yes (RED)
- FOUND: `b440565` feat(02-02): implement gow-yes with 16 KiB prefill buffer (GREEN)

**Build/test gates verified in session output:**

- `cargo build -p gow-true -p gow-false -p gow-yes` → exit 0, zero warnings, zero errors
- `cargo test -p gow-true -p gow-false -p gow-yes` → 20 tests passed, 0 failed
- `cargo clippy -p gow-true -p gow-false -p gow-yes --all-targets -- -D warnings` → exit 0
- `yes.exe | head -c 1000 | wc -c` → 1000 (exact byte count as required)
- `test_broken_pipe_exits_zero` → asserts `status.code() == Some(0)` (green)
- `true.exe` → exit 0 (required by UTIL-08 / D-22)
- `false.exe` → exit 1 (required by UTIL-09 / D-22)

All plan-level success criteria satisfied.

---
*Phase: 02-stateless, Plan: 02*
*Completed: 2026-04-21*
